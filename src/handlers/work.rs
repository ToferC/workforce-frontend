use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::{Identity};
use serde::Deserialize;
use serde_json::json;

use std::sync::Arc;
use crate::{AppData, generate_basic_context, by_lang};
use crate::graphql::{get_work_by_id, all_tasks, create_work, update_work};
use crate::security::{self, MinimumRole};
use super::org_tier::{skill_domain_options, humanize};
use super::task::work_status_options;
use super::capability::CAPABILITY_LEVELS;

fn capability_level_options() -> serde_json::Value {
    json!(CAPABILITY_LEVELS.iter().map(|l| json!({"value": l, "label": humanize(l)})).collect::<Vec<serde_json::Value>>())
}

fn redirect_to(location: String) -> HttpResponse {
    HttpResponse::Found().append_header(("Location", location)).finish()
}

fn csrf_failure_flash(session: &actix_session::Session, lang: &str) {
    security::add_flash(session, "danger", by_lang(lang, "Invalid form token. Please try again.", "Jeton de formulaire invalide. Veuillez réessayer."));
}

#[derive(Deserialize, Debug)]
pub struct WorkForm {
    pub csrf_token: String,
    // Create only (fixed on edit; the API can't move work between tasks/roles)
    #[serde(default)]
    pub task_id: String,
    pub work_description: String,
    #[serde(default)]
    pub url: String,
    pub domain: String,
    pub capability_level: String,
    pub effort: i64,
    pub work_status: String,
}

fn work_from_form(form: &WorkForm, id: Option<&str>) -> serde_json::Value {
    json!({
        "id": id,
        "workDescription": form.work_description,
        "url": form.url,
        "domain": form.domain,
        "capabilityLevel": form.capability_level,
        "effort": form.effort,
        "workStatus": form.work_status,
    })
}

#[get("/{lang}/work/{work_id}")]
pub async fn work_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req:HttpRequest) -> impl Responder {
    let (lang, work_id) = path_params.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_work_by_id(work_id, bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get work");

    ctx.insert("work", &r.work_by_id);

    let rendered = data.tmpl.render("work/work.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// Create a unit of work assigning a role (fixed) to a task (chosen).
#[get("/{lang}/role/{role_id}/work/new")]
pub async fn create_work_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let tasks = all_tasks(auth.bearer, &data.api_url, Arc::clone(&data.client)).await
        .map(|r| json!(r.all_tasks.iter().map(|t| json!({"value": t.id, "label": t.title})).collect::<Vec<_>>()))
        .unwrap_or_else(|_| json!([]));

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &false);
    ctx.insert("role_id", &role_id);
    ctx.insert("work", &json!({"workDescription": "", "url": "", "domain": "", "capabilityLevel": "", "effort": 1, "workStatus": "PLANNING"}));
    ctx.insert("task_options", &tasks);
    ctx.insert("skill_domains", &skill_domain_options());
    ctx.insert("capability_levels", &capability_level_options());
    ctx.insert("work_statuses", &work_status_options());

    let rendered = data.tmpl.render("work/work_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/role/{role_id}/work/new")]
pub async fn create_work_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<WorkForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/role/{}/work/new", &lang, &role_id));
    }

    let new_work = create_work::NewWork {
        task_id: form.task_id.clone(),
        role_id: role_id.clone(),
        work_description: form.work_description.trim().to_string(),
        url: if form.url.trim().is_empty() { None } else { Some(form.url.trim().to_string()) },
        domain: serde_json::from_value(json!(form.domain)).expect("SkillDomain deserialization is infallible"),
        capability_level: serde_json::from_value(json!(form.capability_level)).expect("CapabilityLevel deserialization is infallible"),
        effort: form.effort,
        work_status: serde_json::from_value(json!(form.work_status)).expect("WorkStatus deserialization is infallible"),
    };

    match create_work(new_work, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(&session, "success", by_lang(&lang, "Work created.", "Travail créé."));
            redirect_to(format!("/{}/work/{}", &lang, response.create_work.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            let tasks = all_tasks(String::new(), &data.api_url, Arc::clone(&data.client)).await
                .map(|r| json!(r.all_tasks.iter().map(|t| json!({"value": t.id, "label": t.title})).collect::<Vec<_>>()))
                .unwrap_or_else(|_| json!([]));
            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &false);
            ctx.insert("role_id", &role_id);
            ctx.insert("work", &work_from_form(&form, None));
            ctx.insert("task_options", &tasks);
            ctx.insert("skill_domains", &skill_domain_options());
            ctx.insert("capability_levels", &capability_level_options());
            ctx.insert("work_statuses", &work_status_options());
            let rendered = data.tmpl.render("work/work_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}

#[get("/{lang}/work/{work_id}/edit")]
pub async fn edit_work_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, work_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_work_by_id(work_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &true);
    ctx.insert("work", &r.work_by_id);
    ctx.insert("skill_domains", &skill_domain_options());
    ctx.insert("capability_levels", &capability_level_options());
    ctx.insert("work_statuses", &work_status_options());

    let rendered = data.tmpl.render("work/work_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/work/{work_id}/edit")]
pub async fn edit_work_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<WorkForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, work_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/work/{}/edit", &lang, &work_id));
    }

    let work_data = update_work::WorkData {
        id: work_id.clone(),
        work_description: Some(form.work_description.trim().to_string()),
        url: if form.url.trim().is_empty() { None } else { Some(form.url.trim().to_string()) },
        domain: Some(serde_json::from_value(json!(form.domain)).expect("SkillDomain deserialization is infallible")),
        capability_level: Some(serde_json::from_value(json!(form.capability_level)).expect("CapabilityLevel deserialization is infallible")),
        effort: Some(form.effort),
        work_status: Some(serde_json::from_value(json!(form.work_status)).expect("WorkStatus deserialization is infallible")),
    };

    match update_work(work_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(&session, "success", by_lang(&lang, "Work updated.", "Travail mis à jour."));
            redirect_to(format!("/{}/work/{}", &lang, response.update_work.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &true);
            ctx.insert("work", &work_from_form(&form, Some(&work_id)));
            ctx.insert("skill_domains", &skill_domain_options());
            ctx.insert("capability_levels", &capability_level_options());
            ctx.insert("work_statuses", &work_status_options());
            let rendered = data.tmpl.render("work/work_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}
