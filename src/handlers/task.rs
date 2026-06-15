use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::{Identity};
use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;
use serde_json::json;

use std::sync::Arc;
use crate::{AppData, generate_basic_context, by_lang};
use crate::graphql::{get_task_by_id, all_tasks, create_task, update_task};
use crate::security::{self, MinimumRole};
use super::org_tier::{skill_domain_options, humanize};

/// WorkStatus enum values (used by both tasks and work).
pub const WORK_STATUSES: [&str; 5] = ["PLANNING", "IN_PROGRESS", "COMPLETED", "BLOCKED", "CANCELLED"];

pub fn work_status_options() -> serde_json::Value {
    json!(WORK_STATUSES.iter().map(|s| json!({"value": s, "label": humanize(s)})).collect::<Vec<serde_json::Value>>())
}

pub fn parse_date(value: &str) -> Option<NaiveDateTime> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").ok().and_then(|d| d.and_hms_opt(0, 0, 0))
}

fn redirect_to(location: String) -> HttpResponse {
    HttpResponse::Found().append_header(("Location", location)).finish()
}

fn csrf_failure_flash(session: &actix_session::Session, lang: &str) {
    security::add_flash(session, "danger", by_lang(lang, "Invalid form token. Please try again.", "Jeton de formulaire invalide. Veuillez réessayer."));
}

#[derive(Deserialize, Debug)]
pub struct TaskForm {
    pub csrf_token: String,
    pub title: String,
    pub domain: String,
    pub intended_outcome: String,
    #[serde(default)]
    pub final_outcome: String,
    pub approval_tier: i64,
    pub url: String,
    pub start_date: String,
    pub target_completion_date: String,
    pub task_status: String,
    #[serde(default)]
    pub completed_date: String,
}

fn task_from_form(form: &TaskForm, id: Option<&str>) -> serde_json::Value {
    json!({
        "id": id,
        "title": form.title,
        "domain": form.domain,
        "intendedOutcome": form.intended_outcome,
        "finalOutcome": form.final_outcome,
        "approvalTier": form.approval_tier,
        "url": form.url,
        "startDatestamp": form.start_date,
        "targetCompletionDate": form.target_completion_date,
        "taskStatus": form.task_status,
        "completedDate": form.completed_date,
    })
}

#[get("/{lang}/tasks")]
pub async fn task_index(
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

    let r = all_tasks(bearer, &data.api_url, Arc::clone(&data.client)).await.expect("Unable to get tasks");
    ctx.insert("tasks", &r.all_tasks);

    let rendered = data.tmpl.render("task/task_index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/task/{task_id}")]
pub async fn task_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req:HttpRequest) -> impl Responder {
    let (lang, task_id) = path_params.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_task_by_id(task_id, bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get task");

    ctx.insert("task", &r.task_by_id);

    let rendered = data.tmpl.render("task/task.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// Create a task created by a given role (the role is the creator).
#[get("/{lang}/role/{role_id}/task/new")]
pub async fn create_task_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id) = path_params.into_inner();
    let session = req.get_session();

    if let Err(response) = security::require_role(&session, &lang, MinimumRole::Operator) {
        return response;
    }

    let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &false);
    ctx.insert("role_id", &role_id);
    ctx.insert("task", &json!({
        "title": "", "domain": "", "intendedOutcome": "", "finalOutcome": "", "approvalTier": 1,
        "url": "", "startDatestamp": today, "targetCompletionDate": today, "taskStatus": "PLANNING", "completedDate": "",
    }));
    ctx.insert("skill_domains", &skill_domain_options());
    ctx.insert("work_statuses", &work_status_options());

    let rendered = data.tmpl.render("task/task_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/role/{role_id}/task/new")]
pub async fn create_task_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<TaskForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/role/{}/task/new", &lang, &role_id));
    }

    let new_task = create_task::NewTask {
        created_by_role_id: role_id.clone(),
        title: form.title.trim().to_string(),
        domain: serde_json::from_value(json!(form.domain)).expect("SkillDomain deserialization is infallible"),
        intended_outcome: form.intended_outcome.trim().to_string(),
        approval_tier: form.approval_tier,
        url: form.url.trim().to_string(),
        start_datestamp: parse_date(&form.start_date).unwrap_or_else(|| chrono::Utc::now().naive_utc()),
        target_completion_date: parse_date(&form.target_completion_date).unwrap_or_else(|| chrono::Utc::now().naive_utc()),
        task_status: serde_json::from_value(json!(form.task_status)).expect("WorkStatus deserialization is infallible"),
    };

    match create_task(new_task, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(&session, "success", by_lang(&lang, "Task created.", "Tâche créée."));
            redirect_to(format!("/{}/task/{}", &lang, response.create_task.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &false);
            ctx.insert("role_id", &role_id);
            ctx.insert("task", &task_from_form(&form, None));
            ctx.insert("skill_domains", &skill_domain_options());
            ctx.insert("work_statuses", &work_status_options());
            let rendered = data.tmpl.render("task/task_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}

#[get("/{lang}/task/{task_id}/edit")]
pub async fn edit_task_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, task_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_task_by_id(task_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/tasks", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &true);
    ctx.insert("task", &r.task_by_id);
    ctx.insert("skill_domains", &skill_domain_options());
    ctx.insert("work_statuses", &work_status_options());

    let rendered = data.tmpl.render("task/task_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/task/{task_id}/edit")]
pub async fn edit_task_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<TaskForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, task_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/task/{}/edit", &lang, &task_id));
    }

    let task_data = update_task::TaskData {
        id: task_id.clone(),
        title: Some(form.title.trim().to_string()),
        domain: Some(serde_json::from_value(json!(form.domain)).expect("SkillDomain deserialization is infallible")),
        intended_outcome: Some(form.intended_outcome.trim().to_string()),
        final_outcome: if form.final_outcome.trim().is_empty() { None } else { Some(form.final_outcome.trim().to_string()) },
        url: Some(form.url.trim().to_string()),
        approval_tier: Some(form.approval_tier),
        start_datestamp: parse_date(&form.start_date),
        target_completion_date: parse_date(&form.target_completion_date),
        task_status: Some(serde_json::from_value(json!(form.task_status)).expect("WorkStatus deserialization is infallible")),
        completed_date: parse_date(&form.completed_date),
    };

    match update_task(task_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(&session, "success", by_lang(&lang, "Task updated.", "Tâche mise à jour."));
            redirect_to(format!("/{}/task/{}", &lang, response.update_task.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &true);
            ctx.insert("task", &task_from_form(&form, Some(&task_id)));
            ctx.insert("skill_domains", &skill_domain_options());
            ctx.insert("work_statuses", &work_status_options());
            let rendered = data.tmpl.render("task/task_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}
