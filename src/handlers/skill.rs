use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::{Identity};
use serde::Deserialize;
use serde_json::json;

use std::sync::Arc;
use crate::{AppData, generate_basic_context, by_lang};
use crate::graphql::{all_skills, get_skill_by_id, create_skill, update_skill};
use crate::security::{self, MinimumRole};
use super::org_tier::skill_domain_options;

#[derive(Deserialize, Debug)]
pub struct SkillForm {
    pub csrf_token: String,
    pub name_en: String,
    pub name_fr: String,
    pub description_en: String,
    pub description_fr: String,
    pub domain: String,
}

fn redirect_to(location: String) -> HttpResponse {
    HttpResponse::Found().append_header(("Location", location)).finish()
}

fn csrf_failure_flash(session: &actix_session::Session, lang: &str) {
    security::add_flash(
        session,
        "danger",
        by_lang(lang, "Invalid form token. Please try again.", "Jeton de formulaire invalide. Veuillez réessayer."),
    );
}

fn skill_from_form(form: &SkillForm, id: Option<&str>) -> serde_json::Value {
    json!({
        "id": id,
        "nameEn": form.name_en,
        "nameFr": form.name_fr,
        "descriptionEn": form.description_en,
        "descriptionFr": form.description_fr,
        "domain": form.domain,
    })
}

#[get("/{lang}/skills")]
pub async fn skill_index(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = all_skills(bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get skills");

    ctx.insert("skills", &r.skills);

    let rendered = data.tmpl.render("skill/skill_index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/skill/{skill_id}")]
pub async fn skill_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, skill_id) = path_params.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_skill_by_id(skill_id, bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get skill");

    ctx.insert("skill", &r.skill_by_id);

    let rendered = data.tmpl.render("skill/skill.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/skill/new")]
pub async fn create_skill_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    if let Err(response) = security::require_role(&session, &lang, MinimumRole::Operator) {
        return response;
    }

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &false);
    ctx.insert("skill", &json!({"nameEn": "", "nameFr": "", "descriptionEn": "", "descriptionFr": "", "domain": ""}));
    ctx.insert("skill_domains", &skill_domain_options());

    let rendered = data.tmpl.render("skill/skill_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/skill/new")]
pub async fn create_skill_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    form: web::Form<SkillForm>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/skill/new", &lang));
    }

    let new_skill = create_skill::NewSkill {
        name_en: form.name_en.trim().to_string(),
        name_fr: form.name_fr.trim().to_string(),
        description_en: form.description_en.trim().to_string(),
        description_fr: form.description_fr.trim().to_string(),
        domain: serde_json::from_value(json!(form.domain)).expect("SkillDomain deserialization is infallible"),
    };

    match create_skill(new_skill, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(&session, "success", by_lang(&lang, "Skill created.", "Compétence créée."));
            redirect_to(format!("/{}/skill/{}", &lang, response.create_skill.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &false);
            ctx.insert("skill", &skill_from_form(&form, None));
            ctx.insert("skill_domains", &skill_domain_options());
            let rendered = data.tmpl.render("skill/skill_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}

#[get("/{lang}/skill/{skill_id}/edit")]
pub async fn edit_skill_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, skill_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_skill_by_id(skill_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/skills", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &true);
    ctx.insert("skill", &r.skill_by_id);
    ctx.insert("skill_domains", &skill_domain_options());

    let rendered = data.tmpl.render("skill/skill_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/skill/{skill_id}/edit")]
pub async fn edit_skill_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<SkillForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, skill_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/skill/{}/edit", &lang, &skill_id));
    }

    let skill_data = update_skill::SkillData {
        id: skill_id.clone(),
        name_en: Some(form.name_en.trim().to_string()),
        name_fr: Some(form.name_fr.trim().to_string()),
        domain: Some(serde_json::from_value(json!(form.domain)).expect("SkillDomain deserialization is infallible")),
        description_en: Some(form.description_en.trim().to_string()),
        description_fr: Some(form.description_fr.trim().to_string()),
    };

    match update_skill(skill_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(&session, "success", by_lang(&lang, "Skill updated.", "Compétence mise à jour."));
            redirect_to(format!("/{}/skill/{}", &lang, response.update_skill.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &true);
            ctx.insert("skill", &skill_from_form(&form, Some(&skill_id)));
            ctx.insert("skill_domains", &skill_domain_options());
            let rendered = data.tmpl.render("skill/skill_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}
