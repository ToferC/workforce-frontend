use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::{Identity};
use serde::Deserialize;
use serde_json::json;

use std::sync::Arc;
use crate::{AppData, generate_basic_context, by_lang};
use crate::graphql::{get_people_by_name, get_person_by_id, get_user_by_email, create_person, update_person, all_organizations, create_affiliation, update_affiliation};
use crate::security::{self, MinimumRole};

#[derive(Deserialize, Debug)]
pub struct PersonForm {
    pub csrf_token: String,
    // Create only: the email of the user account this person links to
    #[serde(default)]
    pub user_email: String,
    pub family_name: String,
    pub given_name: String,
    pub email: String,
    pub phone: String,
    pub work_address: String,
    pub city: String,
    pub province: String,
    pub postal_code: String,
    pub country: String,
    pub organization_id: String,
    #[serde(default)]
    pub peoplesoft_id: String,
    #[serde(default)]
    pub orcid_id: String,
}

#[derive(Deserialize, Debug)]
pub struct RetireForm {
    pub csrf_token: String,
}

fn redirect_to(location: String) -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", location))
        .finish()
}

fn csrf_failure_flash(session: &actix_session::Session, lang: &str) {
    security::add_flash(
        session,
        "danger",
        by_lang(lang, "Invalid form token. Please try again.", "Jeton de formulaire invalide. Veuillez réessayer."),
    );
}

pub async fn organization_options(bearer: &str, data: &AppData) -> serde_json::Value {
    match all_organizations(bearer.to_string(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => json!(r.all_organizations
            .iter()
            .map(|org| json!({"value": org.id, "label": org.name_en}))
            .collect::<Vec<serde_json::Value>>()),
        Err(_) => json!([]),
    }
}

/// Resolve a typed "Given Family" name to a person id. The API's
/// personByName does ilike on family OR given name separately, so it
/// can't match a concatenated name; search the last token and filter for
/// an exact full-name match. Returns Ok(None) for blank input,
/// Ok(Some(id)) when resolved, Err(message) when not found / ambiguous.
pub async fn resolve_person_by_name(name: &str, bearer: &str, lang: &str, data: &AppData) -> Result<Option<String>, String> {
    let typed = name.trim().to_string();
    if typed.is_empty() {
        return Ok(None);
    }
    let token = typed.split_whitespace().last().unwrap_or(&typed).to_string();
    match get_people_by_name(token, bearer.to_string(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => {
            let matches = r.person_by_name;
            let exact: Vec<_> = matches
                .iter()
                .filter(|p| format!("{} {}", p.given_name, p.family_name).eq_ignore_ascii_case(&typed))
                .collect();
            if exact.len() == 1 {
                Ok(Some(exact[0].id.clone()))
            } else if exact.len() > 1 {
                Err(by_lang(lang,
                    "Several people share that exact name — assign from the person's page instead.",
                    "Plusieurs personnes portent exactement ce nom — affectez depuis la page de la personne.").to_string())
            } else if matches.len() == 1 {
                Ok(Some(matches[0].id.clone()))
            } else if matches.is_empty() {
                Err(by_lang(lang, "No person found with that name.", "Aucune personne trouvée avec ce nom.").to_string())
            } else {
                Err(by_lang(lang,
                    "Several people match that name — please use the full given and family name.",
                    "Plusieurs personnes correspondent à ce nom — veuillez utiliser le prénom et le nom complets.").to_string())
            }
        },
        Err(e) => Err(e.to_string()),
    }
}

fn person_from_form(form: &PersonForm, id: Option<&str>) -> serde_json::Value {
    json!({
        "id": id,
        "userEmail": form.user_email,
        "familyName": form.family_name,
        "givenName": form.given_name,
        "email": form.email,
        "phone": form.phone,
        "workAddress": form.work_address,
        "city": form.city,
        "province": form.province,
        "postalCode": form.postal_code,
        "country": form.country,
        "peoplesoftId": form.peoplesoft_id,
        "orcidId": form.orcid_id,
        "organization": {"id": form.organization_id},
    })
}

#[get("/{lang}/person_by_name/{name}")]
pub async fn person_by_name(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req:HttpRequest) -> impl Responder {
    let (lang, name) = path_params.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let people = get_people_by_name(name, bearer.clone(), &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get people");

    ctx.insert("people", &people.person_by_name);

    let rendered = data.tmpl.render("person/person_by_name.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}


#[get("/{lang}/person/{person_id}")]
pub async fn person_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req:HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_person_by_id(person_id, bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get person");

    ctx.insert("person", &r.person_by_id);

    let rendered = data.tmpl.render("person/person.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/person/new")]
pub async fn create_person_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &false);
    ctx.insert("person", &json!({
        "userEmail": "", "familyName": "", "givenName": "", "email": "", "phone": "",
        "workAddress": "", "city": "", "province": "", "postalCode": "", "country": "",
        "peoplesoftId": "", "orcidId": "",
        "organization": {"id": ""},
    }));
    ctx.insert("organization_options", &organization_options(&auth.bearer, &data).await);

    let rendered = data.tmpl.render("person/person_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/person/new")]
pub async fn create_person_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    form: web::Form<PersonForm>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/person/new", &lang));
    }

    let render_error = |ctx_error: String, id: Option<Identity>, organization_options: serde_json::Value| {
        // Flash must be queued before generate_basic_context drains the queue
        security::add_flash(&session, "danger", &ctx_error);
        let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
        ctx.insert("edit", &false);
        ctx.insert("person", &person_from_form(&form, None));
        ctx.insert("organization_options", &organization_options);
        let rendered = data.tmpl.render("person/person_form.html", &ctx).unwrap();
        HttpResponse::Ok().body(rendered)
    };

    // A person record must link to an existing user account
    let user = match get_user_by_email(
        form.user_email.to_lowercase().trim().to_string(),
        auth.bearer.clone(),
        &data.api_url,
        Arc::clone(&data.client),
    ).await {
        Ok(r) => r.user_by_email,
        Err(_) => {
            let message = by_lang(
                &lang,
                "No user account found with that email. The person must have a registered user account first.",
                "Aucun compte utilisateur trouvé avec ce courriel. La personne doit d'abord avoir un compte utilisateur.",
            ).to_string();
            let options = organization_options(&auth.bearer, &data).await;
            return render_error(message, id, options);
        },
    };

    let new_person = create_person::NewPerson {
        user_id: user.id,
        family_name: form.family_name.trim().to_string(),
        given_name: form.given_name.trim().to_string(),
        email: form.email.trim().to_string(),
        phone: form.phone.trim().to_string(),
        work_address: form.work_address.trim().to_string(),
        city: form.city.trim().to_string(),
        province: form.province.trim().to_string(),
        postal_code: form.postal_code.trim().to_string(),
        country: form.country.trim().to_string(),
        organization_id: form.organization_id.clone(),
        peoplesoft_id: form.peoplesoft_id.trim().to_string(),
        orcid_id: form.orcid_id.trim().to_string(),
    };

    match create_person(new_person, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Person created.", "Personne créée."),
            );
            redirect_to(format!("/{}/person/{}", &lang, response.create_person.id))
        },
        Err(e) => {
            let options = organization_options(&auth.bearer, &data).await;
            render_error(e.to_string(), id, options)
        },
    }
}

#[get("/{lang}/person/{person_id}/edit")]
pub async fn edit_person_form(
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

    let r = match get_person_by_id(person_id, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &true);
    ctx.insert("person", &r.person_by_id);
    ctx.insert("organization_options", &organization_options(&auth.bearer, &data).await);

    let rendered = data.tmpl.render("person/person_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/person/{person_id}/edit")]
pub async fn edit_person_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<PersonForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/person/{}/edit", &lang, &person_id));
    }

    let person_data = update_person::PersonData {
        id: person_id.clone(),
        user_id: None,
        family_name: Some(form.family_name.trim().to_string()),
        given_name: Some(form.given_name.trim().to_string()),
        email: Some(form.email.trim().to_string()),
        phone: Some(form.phone.trim().to_string()),
        work_address: Some(form.work_address.trim().to_string()),
        city: Some(form.city.trim().to_string()),
        province: Some(form.province.trim().to_string()),
        postal_code: Some(form.postal_code.trim().to_string()),
        country: Some(form.country.trim().to_string()),
        organization_id: Some(form.organization_id.clone()),
        peoplesoft_id: Some(form.peoplesoft_id.trim().to_string()),
        orcid_id: Some(form.orcid_id.trim().to_string()),
        updated_at: None,
        retired_at: None,
    };

    match update_person(person_data, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Person updated.", "Personne mise à jour."),
            );
            redirect_to(format!("/{}/person/{}", &lang, response.update_person.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());

            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &true);
            ctx.insert("person", &person_from_form(&form, Some(&person_id)));
            ctx.insert("organization_options", &organization_options(&auth.bearer, &data).await);

            let rendered = data.tmpl.render("person/person_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}

#[get("/{lang}/person/{person_id}/retire")]
pub async fn retire_person_form(
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

    let r = match get_person_by_id(person_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("person", &r.person_by_id);

    let rendered = data.tmpl.render("person/person_retire.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/person/{person_id}/retire")]
pub async fn retire_person_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<RetireForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/person/{}", &lang, &person_id));
    }

    let person_data = update_person::PersonData {
        id: person_id.clone(),
        user_id: None,
        family_name: None,
        given_name: None,
        email: None,
        phone: None,
        work_address: None,
        city: None,
        province: None,
        postal_code: None,
        country: None,
        organization_id: None,
        peoplesoft_id: None,
        orcid_id: None,
        updated_at: None,
        retired_at: Some(chrono::Utc::now().naive_utc()),
    };

    match update_person(person_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => {
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Person retired.", "Personne retirée."),
            );
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
        },
    };

    redirect_to(format!("/{}/person/{}", &lang, &person_id))
}

#[derive(Deserialize, Debug)]
pub struct AffiliationForm {
    pub csrf_token: String,
    pub organization_id: String,
    pub affiliation_role: String,
    #[serde(default)]
    pub end_date: String,
}

#[derive(Deserialize, Debug)]
pub struct EndAffiliationForm {
    pub csrf_token: String,
}

fn parse_date(value: &str) -> Option<chrono::NaiveDateTime> {
    chrono::NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
}

#[get("/{lang}/person/{person_id}/affiliation/new")]
pub async fn create_affiliation_form(
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

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("person", &person);
    ctx.insert("affiliation", &json!({"organization": {"id": ""}, "affiliationRole": "", "endDate": ""}));
    ctx.insert("organization_options", &organization_options(&auth.bearer, &data).await);

    let rendered = data.tmpl.render("person/affiliation_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/person/{person_id}/affiliation/new")]
pub async fn create_affiliation_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<AffiliationForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/person/{}/affiliation/new", &lang, &person_id));
    }

    // homeOrgId is the person's own organization
    let home_org_id = match get_person_by_id(person_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.person_by_id.organization.id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/person/{}", &lang, &person_id));
        },
    };

    let new_affiliation = create_affiliation::NewAffiliation {
        person_id: person_id.clone(),
        organization_id: form.organization_id.clone(),
        home_org_id,
        affiliation_role: form.affiliation_role.trim().to_string(),
        end_date: parse_date(&form.end_date),
    };

    match create_affiliation(new_affiliation, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Affiliation added.", "Affiliation ajoutée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/person/{}", &lang, &person_id))
}

#[post("/{lang}/person/{person_id}/affiliation/{affiliation_id}/end")]
pub async fn end_affiliation_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String, String)>,
    form: web::Form<EndAffiliationForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id, affiliation_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/person/{}", &lang, &person_id));
    }

    let affiliation_data = update_affiliation::AffiliationData {
        id: affiliation_id,
        affiliation_role: None,
        start_datestamp: None,
        end_date: Some(chrono::Utc::now().naive_utc()),
    };

    match update_affiliation(affiliation_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Affiliation ended.", "Affiliation terminée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/person/{}", &lang, &person_id))
}
