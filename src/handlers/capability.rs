use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::Identity;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use crate::{AppData, generate_basic_context, by_lang};
use crate::graphql::{get_capability_by_name_and_level, get_skill_by_id, get_person_by_id, create_capability, update_capability, create_validation, get_user_by_email};
use crate::security::{self, MinimumRole};
use super::person::resolve_person_by_name;

/// CapabilityLevel enum values, kept in sync with the API schema.
pub const CAPABILITY_LEVELS: [&str; 5] = ["DESIRED", "NOVICE", "EXPERIENCED", "EXPERT", "SPECIALIST"];

fn level_options() -> serde_json::Value {
    json!(CAPABILITY_LEVELS
        .iter()
        .map(|l| json!({"value": l, "label": super::org_tier::humanize(l)}))
        .collect::<Vec<serde_json::Value>>())
}

#[derive(Deserialize, Debug)]
pub struct CapabilityForm {
    pub csrf_token: String,
    pub skill_id: String,
    pub self_identified_level: String,
}

#[derive(Deserialize, Debug)]
pub struct RetireForm {
    pub csrf_token: String,
}

fn redirect_to(location: String) -> HttpResponse {
    HttpResponse::Found().append_header(("Location", location)).finish()
}

#[get("/{lang}/capability_search/{name}/{level}")]
pub async fn capability_search(
    path_params: web::Path<(String, String, String)>,
    data: web::Data<AppData>,
    req: HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    let (lang, name, level) = path_params.into_inner();
    println!("CALL CAPABILITY SEARCH");

    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };
    
    // query graphql API
    let results = get_capability_by_name_and_level(
        name.to_lowercase().trim().to_string(),
        level.clone(),
        bearer.clone(),
        &data.api_url,
        Arc::clone(&data.client),
    )
    .await
    .expect("Unable to find capabilities");

    println!("{:?}", &results);
             
    ctx.insert("capabilities", &results.capabilities_by_name_and_level);
    ctx.insert("name", &name.to_owned());
    ctx.insert("level", &level);

    let rendered = data.tmpl.render("capability/capability_search_results.html", &ctx).unwrap();
    HttpResponse::Ok()
        .body(rendered)
}
#[get("/{lang}/person/{person_id}/capability/new")]
pub async fn create_capability_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let person = match get_person_by_id(person_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.person_by_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let (skill_domains, skill_groups) = super::skill::skill_picker_data(&data, auth.bearer).await;

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("person", &person);
    ctx.insert("skill_domains", &skill_domains);
    ctx.insert("skill_groups", &skill_groups);
    ctx.insert("capability_levels", &level_options());

    let rendered = data.tmpl.render("capability/capability_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/person/{person_id}/capability/new")]
pub async fn create_capability_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<CapabilityForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        security::add_flash(&session, "danger", by_lang(&lang, "Invalid form token. Please try again.", "Jeton de formulaire invalide. Veuillez réessayer."));
        return redirect_to(format!("/{}/person/{}/capability/new", &lang, &person_id));
    }

    // The person supplies the org; the skill supplies name/domain
    let person = match get_person_by_id(person_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.person_by_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/person/{}", &lang, &person_id));
        },
    };
    let skill = match get_skill_by_id(form.skill_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.skill_by_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/person/{}/capability/new", &lang, &person_id));
        },
    };

    let new_capability = create_capability::NewCapability {
        name_en: skill.name_en.clone(),
        name_fr: skill.name_fr.clone(),
        domain: serde_json::from_value(json!(skill.domain)).expect("SkillDomain deserialization is infallible"),
        person_id: person_id.clone(),
        skill_id: form.skill_id.clone(),
        organization_id: person.organization.id.clone(),
        self_identified_level: serde_json::from_value(json!(form.self_identified_level)).expect("CapabilityLevel deserialization is infallible"),
    };

    match create_capability(new_capability, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Capability added.", "Capacité ajoutée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/person/{}", &lang, &person_id))
}

#[post("/{lang}/person/{person_id}/capability/{capability_id}/retire")]
pub async fn retire_capability_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String, String)>,
    form: web::Form<RetireForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id, capability_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        security::add_flash(&session, "danger", by_lang(&lang, "Invalid form token. Please try again.", "Jeton de formulaire invalide. Veuillez réessayer."));
        return redirect_to(format!("/{}/person/{}", &lang, &person_id));
    }

    let capability_data = update_capability::CapabilityData {
        id: capability_id,
        self_identified_level: None,
        validated_level: None,
        retired_at: Some(chrono::Utc::now().naive_utc()),
    };

    match update_capability(capability_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Capability retired.", "Capacité retirée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/person/{}", &lang, &person_id))
}

#[derive(Deserialize, Debug)]
pub struct ValidationForm {
    pub csrf_token: String,
    pub validated_level: String,
}

/// Admin-only: validate someone's capability. The currently signed-in user
/// is automatically recorded as the validating authority. The submitted level
/// is set directly as the capability's validated level (latest authoritative
/// validation wins); it also stamps who validated it and when.
#[get("/{lang}/person/{person_id}/capability/{capability_id}/validate")]
pub async fn validate_capability_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id, capability_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Admin) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let validator_display = match session.get::<String>("session_user").ok().flatten() {
        Some(email) => match get_user_by_email(email, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
            Ok(r) => r.user_by_email.name,
            Err(_) => by_lang(&lang, "(unknown)", "(inconnu)").to_string(),
        },
        None => by_lang(&lang, "(unknown)", "(inconnu)").to_string(),
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("person_id", &person_id);
    ctx.insert("capability_id", &capability_id);
    ctx.insert("capability_levels", &level_options());
    ctx.insert("validator_display", &validator_display);

    let rendered = data.tmpl.render("capability/validation_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/person/{person_id}/capability/{capability_id}/validate")]
pub async fn validate_capability_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String, String)>,
    form: web::Form<ValidationForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id, capability_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Admin) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        security::add_flash(&session, "danger", by_lang(&lang, "Invalid form token. Please try again.", "Jeton de formulaire invalide. Veuillez réessayer."));
        return redirect_to(format!("/{}/person/{}/capability/{}/validate", &lang, &person_id, &capability_id));
    }

    let session_email = match session.get::<String>("session_user").ok().flatten() {
        Some(e) => e,
        None => {
            security::add_flash(&session, "danger", by_lang(&lang, "Could not identify the signed-in user.", "Impossible d'identifier l'utilisateur connecté."));
            return redirect_to(format!("/{}/person/{}", &lang, &person_id));
        },
    };

    let user_name = match get_user_by_email(session_email, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.user_by_email.name,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/person/{}", &lang, &person_id));
        },
    };

    match resolve_person_by_name(&user_name, &auth.bearer, &lang, &data).await {
        Ok(Some(validator_id)) => {
            let new_validation = create_validation::NewValidation {
                validator_id,
                capability_id: capability_id.clone(),
                validated_level: serde_json::from_value(json!(form.validated_level)).expect("CapabilityLevel deserialization is infallible"),
            };
            match create_validation(new_validation, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
                Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Validation recorded.", "Validation enregistrée.")),
                Err(e) => security::add_flash(&session, "danger", &e.to_string()),
            };
        },
        Ok(None) => security::add_flash(&session, "danger", by_lang(&lang, "Your account is not linked to a person record.", "Votre compte n'est pas lié à une fiche de personne.")),
        Err(message) => security::add_flash(&session, "danger", &message),
    };

    redirect_to(format!("/{}/person/{}", &lang, &person_id))
}
