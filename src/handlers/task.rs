use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::{Identity};
use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;
use serde_json::json;

use std::sync::Arc;
use crate::{AppData, generate_basic_context, by_lang};
use crate::graphql::{get_task_by_id, all_tasks, create_task, update_task, get_product_by_id, get_team_by_id, submit_task_for_approval, approve_task, reject_task, pending_approvals};
use crate::security::{self, MinimumRole};
use super::org_tier::{skill_domain_options, humanize};
use super::product::product_options;
use super::team::team_role_options;

/// WorkStatus enum values (used by both tasks and work).
pub const WORK_STATUSES: [&str; 5] = ["PLANNING", "IN_PROGRESS", "COMPLETED", "BLOCKED", "CANCELLED"];

pub fn work_status_options() -> serde_json::Value {
    json!(WORK_STATUSES.iter().map(|s| json!({"value": s, "label": humanize(s)})).collect::<Vec<serde_json::Value>>())
}

pub const PRIORITIES: [&str; 4] = ["LOW", "MEDIUM", "HIGH", "CRITICAL"];

pub fn priority_options() -> serde_json::Value {
    json!(PRIORITIES.iter().map(|s| json!({"value": s, "label": humanize(s)})).collect::<Vec<serde_json::Value>>())
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
    pub priority: String,
    #[serde(default)]
    pub completed_date: String,
    #[serde(default)]
    pub product_id: String,
    // Set only by the team-scoped create form, where the creating role is
    // chosen from the team's roles instead of taken from the URL.
    #[serde(default)]
    pub created_by_role_id: String,
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
        "priority": form.priority,
        "completedDate": form.completed_date,
        "productId": form.product_id,
        "createdByRoleId": form.created_by_role_id,
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

    let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &false);
    ctx.insert("role_id", &role_id);
    ctx.insert("task", &json!({
        "title": "", "domain": "", "intendedOutcome": "", "finalOutcome": "", "approvalTier": 1,
        "url": "", "startDatestamp": today, "targetCompletionDate": today, "taskStatus": "PLANNING", "priority": "MEDIUM", "completedDate": "", "productId": "",
    }));
    ctx.insert("skill_domains", &skill_domain_options());
    ctx.insert("work_statuses", &work_status_options());
    ctx.insert("priorities", &priority_options());
    ctx.insert("product_options", &product_options(&auth.bearer, &data).await);

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
        priority: serde_json::from_value(json!(form.priority)).expect("Priority deserialization is infallible"),
        product_id: if form.product_id.trim().is_empty() { None } else { Some(form.product_id.clone()) },
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
            ctx.insert("product_options", &product_options("", &data).await);
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

    let (task_res, product_opts) = futures::join!(
        get_task_by_id(task_id, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)),
        product_options(&auth.bearer, &data),
    );
    let r = match task_res {
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
    ctx.insert("priorities", &priority_options());
    ctx.insert("product_options", &product_opts);

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
        priority: Some(serde_json::from_value(json!(form.priority)).expect("Priority deserialization is infallible")),
        completed_date: parse_date(&form.completed_date),
        product_id: if form.product_id.trim().is_empty() { None } else { Some(form.product_id.clone()) },
    };

    match update_task(task_data, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
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
            ctx.insert("product_options", &product_options(&auth.bearer, &data).await);
            let rendered = data.tmpl.render("task/task_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}

/// Show the task creation form pre-filled for a specific product. The form
/// POSTs to the existing role-scoped task creation handler using the product
/// owner's role as the creator.
#[get("/{lang}/product/{product_id}/task/new")]
pub async fn create_product_task_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, product_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let (product_res, product_opts) = futures::join!(
        get_product_by_id(product_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)),
        product_options(&auth.bearer, &data),
    );
    let product = match product_res {
        Ok(r) => r.product_by_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/products", &lang));
        },
    };

    let role_id = product.product_owner.id.clone();
    let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &false);
    ctx.insert("role_id", &role_id);
    ctx.insert("form_action", &format!("/{}/role/{}/task/new", &lang, &role_id));
    ctx.insert("cancel_url", &format!("/{}/product/{}", &lang, &product_id));
    ctx.insert("task", &json!({
        "title": "", "domain": "", "intendedOutcome": "", "finalOutcome": "", "approvalTier": 1,
        "url": "", "startDatestamp": today, "targetCompletionDate": today, "taskStatus": "PLANNING", "priority": "MEDIUM",
        "completedDate": "", "productId": product_id,
    }));
    ctx.insert("skill_domains", &skill_domain_options());
    ctx.insert("work_statuses", &work_status_options());
    ctx.insert("priorities", &priority_options());
    ctx.insert("product_options", &product_opts);

    let rendered = data.tmpl.render("task/task_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// Show the task creation form scoped to a team. The creating role is chosen
/// from a dropdown of the team's own roles (occupied and vacant) rather than
/// taken from the URL.
#[get("/{lang}/team/{team_id}/task/new")]
pub async fn create_team_task_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, team_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let (team_res, product_opts) = futures::join!(
        get_team_by_id(team_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)),
        product_options(&auth.bearer, &data),
    );
    let team = match team_res {
        Ok(r) => serde_json::to_value(&r.team_by_id).unwrap_or_else(|_| json!({})),
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/team/{}", &lang, &team_id));
        },
    };

    let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &false);
    ctx.insert("role_options", &team_role_options(&team));
    ctx.insert("form_action", &format!("/{}/team/{}/task/new", &lang, &team_id));
    ctx.insert("cancel_url", &format!("/{}/team/{}", &lang, &team_id));
    ctx.insert("task", &json!({
        "title": "", "domain": "", "intendedOutcome": "", "finalOutcome": "", "approvalTier": 1,
        "url": "", "startDatestamp": today, "targetCompletionDate": today, "taskStatus": "PLANNING", "priority": "MEDIUM",
        "completedDate": "", "productId": "", "createdByRoleId": "",
    }));
    ctx.insert("skill_domains", &skill_domain_options());
    ctx.insert("work_statuses", &work_status_options());
    ctx.insert("priorities", &priority_options());
    ctx.insert("product_options", &product_opts);

    let rendered = data.tmpl.render("task/task_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/team/{team_id}/task/new")]
pub async fn create_team_task_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<TaskForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, team_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/team/{}/task/new", &lang, &team_id));
    }

    if form.created_by_role_id.trim().is_empty() {
        security::add_flash(&session, "danger", by_lang(&lang, "Select a creating role on this team.", "Sélectionnez un rôle créateur dans cette équipe."));
        return redirect_to(format!("/{}/team/{}/task/new", &lang, &team_id));
    }

    let new_task = create_task::NewTask {
        created_by_role_id: form.created_by_role_id.clone(),
        title: form.title.trim().to_string(),
        domain: serde_json::from_value(json!(form.domain)).expect("SkillDomain deserialization is infallible"),
        intended_outcome: form.intended_outcome.trim().to_string(),
        approval_tier: form.approval_tier,
        url: form.url.trim().to_string(),
        start_datestamp: parse_date(&form.start_date).unwrap_or_else(|| chrono::Utc::now().naive_utc()),
        target_completion_date: parse_date(&form.target_completion_date).unwrap_or_else(|| chrono::Utc::now().naive_utc()),
        task_status: serde_json::from_value(json!(form.task_status)).expect("WorkStatus deserialization is infallible"),
        priority: serde_json::from_value(json!(form.priority)).expect("Priority deserialization is infallible"),
        product_id: if form.product_id.trim().is_empty() { None } else { Some(form.product_id.clone()) },
    };

    match create_task(new_task, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(&session, "success", by_lang(&lang, "Task created.", "Tâche créée."));
            redirect_to(format!("/{}/task/{}", &lang, response.create_task.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            let (team_res, product_opts) = futures::join!(
                get_team_by_id(team_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)),
                product_options(&auth.bearer, &data),
            );
            let team = team_res
                .map(|r| serde_json::to_value(&r.team_by_id).unwrap_or_else(|_| json!({})))
                .unwrap_or_else(|_| json!({}));
            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &false);
            ctx.insert("role_options", &team_role_options(&team));
            ctx.insert("form_action", &format!("/{}/team/{}/task/new", &lang, &team_id));
            ctx.insert("cancel_url", &format!("/{}/team/{}", &lang, &team_id));
            ctx.insert("task", &task_from_form(&form, None));
            ctx.insert("skill_domains", &skill_domain_options());
            ctx.insert("work_statuses", &work_status_options());
            ctx.insert("priorities", &priority_options());
            ctx.insert("product_options", &product_opts);
            let rendered = data.tmpl.render("task/task_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}

// ── Approval workflow (Proposal 7b) ──────────────────────────────────────────

/// CSRF-only form for the submit / approve actions.
#[derive(Deserialize, Debug)]
pub struct ApprovalActionForm {
    pub csrf_token: String,
    #[serde(default)]
    pub return_to: String,
}

/// Reject form: CSRF + a required reason.
#[derive(Deserialize, Debug)]
pub struct RejectForm {
    pub csrf_token: String,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub return_to: String,
}

fn approval_redirect(return_to: &str, lang: &str, task_id: &str) -> HttpResponse {
    if return_to.starts_with('/') {
        redirect_to(return_to.to_string())
    } else {
        redirect_to(format!("/{}/task/{}", lang, task_id))
    }
}

#[post("/{lang}/task/{task_id}/submit_approval")]
pub async fn submit_task_approval_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<ApprovalActionForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, task_id) = path_params.into_inner();
    let session = req.get_session();
    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };
    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/task/{}", &lang, &task_id));
    }
    match submit_task_for_approval(task_id.clone(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Task submitted for approval.", "Tâche soumise pour approbation.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    }
    approval_redirect(&form.return_to, &lang, &task_id)
}

#[post("/{lang}/task/{task_id}/approve")]
pub async fn approve_task_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<ApprovalActionForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, task_id) = path_params.into_inner();
    let session = req.get_session();
    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };
    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/task/{}", &lang, &task_id));
    }
    match approve_task(task_id.clone(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Task approved.", "Tâche approuvée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    }
    approval_redirect(&form.return_to, &lang, &task_id)
}

#[post("/{lang}/task/{task_id}/reject")]
pub async fn reject_task_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<RejectForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, task_id) = path_params.into_inner();
    let session = req.get_session();
    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };
    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/task/{}", &lang, &task_id));
    }
    if form.reason.trim().is_empty() {
        security::add_flash(&session, "danger", by_lang(&lang, "A rejection reason is required.", "Un motif de rejet est requis."));
        return redirect_to(format!("/{}/task/{}", &lang, &task_id));
    }
    match reject_task(task_id.clone(), form.reason.trim().to_string(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Task rejected.", "Tâche rejetée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    }
    approval_redirect(&form.return_to, &lang, &task_id)
}

/// Approver queue (Proposal 7b): tasks awaiting the operator/admin's approval.
#[get("/{lang}/approvals")]
pub async fn approvals_queue(
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
    let tasks = pending_approvals(Some(200), auth.bearer, &data.api_url, Arc::clone(&data.client)).await
        .map(|r| r.pending_approvals)
        .unwrap_or_default();
    ctx.insert("approval_count", &(tasks.len() as i64));
    ctx.insert("tasks", &tasks);
    let rendered = data.tmpl.render("task/approvals_queue.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}
