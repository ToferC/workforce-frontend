use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::Identity;
use serde::Deserialize;
use serde_json::json;

use std::sync::Arc;
use crate::{AppData, generate_basic_context, by_lang};
use crate::graphql::{get_work_by_id, all_work, all_tasks, all_skills, create_work, update_work, vacant_roles, all_roles, get_me, get_task_by_id, get_team_by_id, my_work, add_work_update, resolve_work_update_flag, open_work_flags};
use crate::security::{self, MinimumRole};
use super::org_tier::{skill_domain_options, humanize};
use super::task::{work_status_options, priority_options, parse_date};
use super::team::team_role_options;
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
    pub priority: String,
    #[serde(default)]
    pub skill_id: String,
    // Set only by the task-scoped create form, where the work may optionally
    // be assigned to a role on the task's team (blank = leave unassigned).
    #[serde(default)]
    pub role_id: String,
    // Proposal 1 — target completion date for this item (YYYY-MM-DD; blank = none).
    #[serde(default)]
    pub due_date: String,
    // Proposal 2 — blocked context; only meaningful when work_status is BLOCKED.
    #[serde(default)]
    pub blocked_reason: String,
    #[serde(default)]
    pub blocked_on_role_id: String,
}

#[derive(Deserialize, Debug)]
pub struct AssignWorkForm {
    pub csrf_token: String,
    pub role_id: String,
    #[serde(default)]
    pub skill_id: String,
}

/// Proposal 3 — posting a comment or flag on a work item.
#[derive(Deserialize, Debug)]
pub struct WorkUpdateForm {
    pub csrf_token: String,
    pub body: String,
    // "COMMENT" (default) or "FLAG".
    #[serde(default)]
    pub kind: String,
}

/// Minimal CSRF form for the resolve-flag button. `return_to` lets the manager
/// stay on the flags queue after resolving instead of bouncing to the work page.
#[derive(Deserialize, Debug)]
pub struct CsrfOnlyForm {
    pub csrf_token: String,
    #[serde(default)]
    pub return_to: String,
}

/// SkillDomain key (e.g. "SOFTWARE_ENGINEERING") for a generated enum value.
fn domain_key(domain: &impl serde::Serialize) -> String {
    serde_json::to_value(domain)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

/// Build a JSON {value, label} list of active skills in a single domain, for
/// the Work form's Required Skill select. Required domain and level are
/// chosen first; this narrows Skill to that domain instead of offering every
/// skill across every domain. Returns an empty list if no domain is chosen.
async fn skill_options_for_domain(domain: &str, bearer: &str, data: &AppData) -> serde_json::Value {
    if domain.is_empty() {
        return json!([]);
    }
    all_skills(bearer.to_string(), &data.api_url, Arc::clone(&data.client)).await
        .map(|r| {
            let mut matched: Vec<_> = r.skills.iter()
                .filter(|s| s.retired_at.is_none())
                .filter(|s| domain_key(&s.domain) == domain)
                .collect();
            matched.sort_by(|a, b| a.name_en.to_lowercase().cmp(&b.name_en.to_lowercase()));
            json!(matched.iter().map(|s| json!({"value": s.id, "label": s.name_en})).collect::<Vec<_>>())
        })
        .unwrap_or_else(|_| json!([]))
}

/// Resolve {value,label} role options for the team that owns a task, via the
/// task's creating role. Lets task-scoped work be assigned to a role on the
/// same team. Returns an empty list if anything in the chain is unavailable.
async fn task_team_role_options(task_id: &str, bearer: &str, data: &AppData) -> serde_json::Value {
    let team_id = match get_task_by_id(task_id.to_string(), bearer.to_string(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.task_by_id.created_by.team.id,
        Err(_) => return json!([]),
    };
    match get_team_by_id(team_id, bearer.to_string(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => team_role_options(&serde_json::to_value(&r.team_by_id).unwrap_or_else(|_| json!({}))),
        Err(_) => json!([]),
    }
}

/// A work item is overdue when its due date has passed and it is still open
/// (not completed or cancelled). Used to badge late work (Proposal 1).
fn is_overdue(due_date: Option<chrono::NaiveDateTime>, status: &str) -> bool {
    match due_date {
        Some(d) => d.date() < chrono::Utc::now().date_naive()
            && status != "COMPLETED" && status != "CANCELLED",
        None => false,
    }
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
        "priority": form.priority,
        "dueDate": form.due_date,
        "blockedReason": form.blocked_reason,
        // Preserve the chosen "waiting on" role across a failed submit.
        "blockedOnRole": if form.blocked_on_role_id.trim().is_empty() {
            serde_json::Value::Null
        } else {
            json!({"id": form.blocked_on_role_id})
        },
    })
}

#[derive(Deserialize, Debug)]
pub struct WorkSkillOptionsParams {
    #[serde(default)]
    pub domain: String,
}

/// HTMX partial: the Required Skill select scoped to a single domain. Loaded
/// by the Work form's Domain select (see templates/work/_skill_select.html)
/// and swapped into #skill-select-wrapper whenever the domain changes.
#[get("/{lang}/work_skill_options")]
pub async fn work_skill_options(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    params: web::Query<WorkSkillOptionsParams>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match session.get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let domain = params.domain.trim().to_string();
    let skills = skill_options_for_domain(&domain, &bearer, &data).await;

    ctx.insert("domain", &domain);
    ctx.insert("skill_id", &"");
    ctx.insert("skill_options", &skills);

    let rendered = data.tmpl.render("work/_skill_select.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/work/{work_id}")]
pub async fn work_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
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

    let status = domain_key(&r.work_by_id.work_status);
    ctx.insert("work_overdue", &is_overdue(r.work_by_id.due_date, &status));
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

    let (tasks_res, skills) = futures::join!(
        all_tasks(auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)),
        skill_options_for_domain("", &auth.bearer, &data),
    );
    let tasks = tasks_res
        .map(|r| json!(r.all_tasks.iter().map(|t| json!({"value": t.id, "label": t.title})).collect::<Vec<_>>()))
        .unwrap_or_else(|_| json!([]));

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &false);
    ctx.insert("vacant", &false);
    ctx.insert("role_id", &role_id);
    ctx.insert("skill_id", &"");
    ctx.insert("domain", &"");
    ctx.insert("work", &json!({"workDescription": "", "url": "", "domain": "", "capabilityLevel": "", "effort": 1, "workStatus": "PLANNING", "priority": "MEDIUM", "dueDate": ""}));
    ctx.insert("task_options", &tasks);
    ctx.insert("skill_options", &skills);
    ctx.insert("skill_domains", &skill_domain_options());
    ctx.insert("capability_levels", &capability_level_options());
    ctx.insert("work_statuses", &work_status_options());
    ctx.insert("priorities", &priority_options());

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

    if form.skill_id.is_empty() {
        security::add_flash(&session, "danger", by_lang(&lang, "Please select a skill for this work.", "Veuillez sélectionner une compétence pour ce travail."));
        return redirect_to(format!("/{}/role/{}/work/new", &lang, &role_id));
    }

    let new_work = create_work::NewWork {
        task_id: form.task_id.clone(),
        role_id: Some(role_id.clone()),
        skill_id: form.skill_id.clone(),
        work_description: form.work_description.trim().to_string(),
        url: if form.url.trim().is_empty() { None } else { Some(form.url.trim().to_string()) },
        domain: serde_json::from_value(json!(form.domain)).expect("SkillDomain deserialization is infallible"),
        capability_level: serde_json::from_value(json!(form.capability_level)).expect("CapabilityLevel deserialization is infallible"),
        effort: form.effort,
        work_status: serde_json::from_value(json!(form.work_status)).expect("WorkStatus deserialization is infallible"),
        priority: serde_json::from_value(json!(form.priority)).expect("Priority deserialization is infallible"),
        due_date: parse_date(&form.due_date),
    };

    match create_work(new_work, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(&session, "success", by_lang(&lang, "Work created.", "Travail créé."));
            redirect_to(format!("/{}/work/{}", &lang, response.create_work.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            let (tasks_res, skills) = futures::join!(
                all_tasks(auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)),
                skill_options_for_domain(&form.domain, &auth.bearer, &data),
            );
            let tasks = tasks_res
                .map(|r| json!(r.all_tasks.iter().map(|t| json!({"value": t.id, "label": t.title})).collect::<Vec<_>>()))
                .unwrap_or_else(|_| json!([]));
            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &false);
            ctx.insert("vacant", &false);
            ctx.insert("role_id", &role_id);
            ctx.insert("skill_id", &form.skill_id);
            ctx.insert("domain", &form.domain);
            ctx.insert("work", &work_from_form(&form, None));
            ctx.insert("task_options", &tasks);
            ctx.insert("skill_options", &skills);
            ctx.insert("skill_domains", &skill_domain_options());
            ctx.insert("capability_levels", &capability_level_options());
            ctx.insert("work_statuses", &work_status_options());
            ctx.insert("priorities", &priority_options());
            let rendered = data.tmpl.render("work/work_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}

/// Create a vacant work item under a task (no role assigned yet).
#[get("/{lang}/task/{task_id}/work/new")]
pub async fn create_vacant_work_form(
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

    let (skills, role_options) = futures::join!(
        skill_options_for_domain("", &auth.bearer, &data),
        task_team_role_options(&task_id, &auth.bearer, &data),
    );

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &false);
    ctx.insert("vacant", &true);
    ctx.insert("task_id", &task_id);
    ctx.insert("skill_id", &"");
    ctx.insert("domain", &"");
    ctx.insert("role_id", &"");
    ctx.insert("role_options", &role_options);
    ctx.insert("work", &json!({"workDescription": "", "url": "", "domain": "", "capabilityLevel": "", "effort": 1, "workStatus": "PLANNING", "priority": "MEDIUM", "dueDate": ""}));
    ctx.insert("skill_options", &skills);
    ctx.insert("skill_domains", &skill_domain_options());
    ctx.insert("capability_levels", &capability_level_options());
    ctx.insert("work_statuses", &work_status_options());
    ctx.insert("priorities", &priority_options());

    let rendered = data.tmpl.render("work/work_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/task/{task_id}/work/new")]
pub async fn create_vacant_work_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<WorkForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, task_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/task/{}/work/new", &lang, &task_id));
    }

    if form.skill_id.is_empty() {
        security::add_flash(&session, "danger", by_lang(&lang, "Please select a skill for this work.", "Veuillez sélectionner une compétence pour ce travail."));
        return redirect_to(format!("/{}/task/{}/work/new", &lang, &task_id));
    }

    let new_work = create_work::NewWork {
        task_id: task_id.clone(),
        // Optional: assign to a role on the task's team, or leave unassigned.
        role_id: if form.role_id.trim().is_empty() { None } else { Some(form.role_id.clone()) },
        skill_id: form.skill_id.clone(),
        work_description: form.work_description.trim().to_string(),
        url: if form.url.trim().is_empty() { None } else { Some(form.url.trim().to_string()) },
        domain: serde_json::from_value(json!(form.domain)).expect("SkillDomain deserialization is infallible"),
        capability_level: serde_json::from_value(json!(form.capability_level)).expect("CapabilityLevel deserialization is infallible"),
        effort: form.effort,
        work_status: serde_json::from_value(json!(form.work_status)).expect("WorkStatus deserialization is infallible"),
        priority: serde_json::from_value(json!(form.priority)).expect("Priority deserialization is infallible"),
        due_date: parse_date(&form.due_date),
    };

    match create_work(new_work, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(&session, "success", by_lang(&lang, "Work created.", "Travail créé."));
            redirect_to(format!("/{}/work/{}", &lang, response.create_work.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            let (skills, role_options) = futures::join!(
                skill_options_for_domain(&form.domain, &auth.bearer, &data),
                task_team_role_options(&task_id, &auth.bearer, &data),
            );
            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &false);
            ctx.insert("vacant", &true);
            ctx.insert("task_id", &task_id);
            ctx.insert("skill_id", &form.skill_id);
            ctx.insert("domain", &form.domain);
            ctx.insert("role_id", &form.role_id);
            ctx.insert("role_options", &role_options);
            ctx.insert("work", &work_from_form(&form, None));
            ctx.insert("skill_options", &skills);
            ctx.insert("skill_domains", &skill_domain_options());
            ctx.insert("capability_levels", &capability_level_options());
            ctx.insert("work_statuses", &work_status_options());
            ctx.insert("priorities", &priority_options());
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

    let r = match get_work_by_id(work_id, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let current_skill_id = r.work_by_id.skill.id.clone();
    let current_domain = domain_key(&r.work_by_id.domain);
    // The "waiting on" picker for the BLOCKED reveal lists all active roles so
    // a blocker can point at any position across the org (Proposal 2).
    let (skills, blocked_role_options) = futures::join!(
        skill_options_for_domain(&current_domain, &auth.bearer, &data),
        super::product::role_options(&auth.bearer, &data),
    );

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &true);
    ctx.insert("skill_id", &current_skill_id);
    ctx.insert("domain", &current_domain);
    ctx.insert("work", &r.work_by_id);
    ctx.insert("skill_options", &skills);
    ctx.insert("blocked_role_options", &blocked_role_options);
    ctx.insert("skill_domains", &skill_domain_options());
    ctx.insert("capability_levels", &capability_level_options());
    ctx.insert("work_statuses", &work_status_options());
    ctx.insert("priorities", &priority_options());

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
        task_id: None,
        role_id: None,
        skill_id: if form.skill_id.is_empty() { None } else { Some(form.skill_id.clone()) },
        work_description: Some(form.work_description.trim().to_string()),
        url: if form.url.trim().is_empty() { None } else { Some(form.url.trim().to_string()) },
        domain: Some(serde_json::from_value(json!(form.domain)).expect("SkillDomain deserialization is infallible")),
        capability_level: Some(serde_json::from_value(json!(form.capability_level)).expect("CapabilityLevel deserialization is infallible")),
        effort: Some(form.effort),
        work_status: Some(serde_json::from_value(json!(form.work_status)).expect("WorkStatus deserialization is infallible")),
        priority: Some(serde_json::from_value(json!(form.priority)).expect("Priority deserialization is infallible")),
        due_date: parse_date(&form.due_date),
        // Blocked context is only meaningful while BLOCKED; the API also clears
        // it automatically when the work leaves BLOCKED.
        blocked_reason: if form.blocked_reason.trim().is_empty() { None } else { Some(form.blocked_reason.trim().to_string()) },
        blocked_on_role_id: if form.blocked_on_role_id.trim().is_empty() { None } else { Some(form.blocked_on_role_id.clone()) },
    };

    match update_work(work_data, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(&session, "success", by_lang(&lang, "Work updated.", "Travail mis à jour."));
            redirect_to(format!("/{}/work/{}", &lang, response.update_work.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            let (skills, blocked_role_options) = futures::join!(
                skill_options_for_domain(&form.domain, &auth.bearer, &data),
                super::product::role_options(&auth.bearer, &data),
            );
            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &true);
            ctx.insert("skill_id", &form.skill_id);
            ctx.insert("domain", &form.domain);
            ctx.insert("work", &work_from_form(&form, Some(&work_id)));
            ctx.insert("skill_options", &skills);
            ctx.insert("blocked_role_options", &blocked_role_options);
            ctx.insert("skill_domains", &skill_domain_options());
            ctx.insert("capability_levels", &capability_level_options());
            ctx.insert("work_statuses", &work_status_options());
            ctx.insert("priorities", &priority_options());
            let rendered = data.tmpl.render("work/work_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}

/// Show a form to assign (or reassign) work to a role.
#[get("/{lang}/work/{work_id}/assign")]
pub async fn assign_work_form(
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

    let work = match get_work_by_id(work_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.work_by_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    // Skill choices are scoped to the work's (fixed) domain — the remaining
    // two calls are independent, so issue them concurrently.
    let work_domain = domain_key(&work.domain);
    let (skill_opts, me_res, roles_res) = futures::join!(
        skill_options_for_domain(&work_domain, &auth.bearer, &data),
        get_me(auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)),
        all_roles(auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)),
    );

    let current_role_id = work.role.as_ref().map(|r| r.id.clone()).unwrap_or_default();
    let current_skill_id = work.skill.id.clone();

    // The operator/admin's own team(s) — roles here are surfaced first so the
    // default action is to assign work to a role on their own team.
    let my_team_ids: std::collections::HashSet<String> =
        match me_res {
            Ok(r) => r.me.person
                .map(|p| p.active_roles.iter().map(|ar| ar.team.id.clone()).collect())
                .unwrap_or_default(),
            Err(_) => std::collections::HashSet::new(),
        };

    // Build role options once, split into "your team" and "all roles".
    let (team_role_opts, role_opts) = match roles_res {
        Ok(r) => {
            let mut team: Vec<serde_json::Value> = Vec::new();
            let mut all: Vec<serde_json::Value> = Vec::new();
            for role in &r.all_roles {
                let person_prefix = role.person.as_ref()
                    .map(|p| format!("{} {} \u{2014} ", p.given_name, p.family_name))
                    .unwrap_or_else(|| "Vacant \u{2014} ".to_string());
                let opt = json!({
                    "value": role.id,
                    "label": format!("{}{} ({})", person_prefix, role.title_english, role.team.name_english),
                });
                if my_team_ids.contains(&role.team.id) {
                    team.push(opt.clone());
                }
                all.push(opt);
            }
            (json!(team), json!(all))
        },
        Err(_) => (json!([]), json!([])),
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("work", &work);
    ctx.insert("current_role_id", &current_role_id);
    ctx.insert("current_skill_id", &current_skill_id);
    ctx.insert("team_role_options", &team_role_opts);
    ctx.insert("role_options", &role_opts);
    ctx.insert("skill_options", &skill_opts);

    let rendered = data.tmpl.render("work/assign_work.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/work/{work_id}/assign")]
pub async fn assign_work_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<AssignWorkForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, work_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/work/{}/assign", &lang, &work_id));
    }

    let work_data = update_work::WorkData {
        id: work_id.clone(),
        task_id: None,
        role_id: if form.role_id.is_empty() { None } else { Some(form.role_id.clone()) },
        skill_id: if form.skill_id.is_empty() { None } else { Some(form.skill_id.clone()) },
        work_description: None,
        url: None,
        domain: None,
        capability_level: None,
        effort: None,
        work_status: None,
        priority: None,
        // Assignment leaves dates and blocked context untouched.
        due_date: None,
        blocked_reason: None,
        blocked_on_role_id: None,
    };

    match update_work(work_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(&session, "success", by_lang(&lang, "Work assigned.", "Travail assigné."));
            redirect_to(format!("/{}/work/{}", &lang, response.update_work.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            redirect_to(format!("/{}/work/{}/assign", &lang, &work_id))
        },
    }
}

#[derive(Deserialize, Debug)]
pub struct WorkIndexParams {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub unassigned: String,
    #[serde(default)]
    pub page: String,
}

/// Work items shown per page. The API now filters and paginates server-side.
const WORK_PAGE_SIZE: i64 = 50;

/// Index of all work items, with optional filtering by status and by
/// "unassigned only". Filtering is applied template-side from the full list.
#[get("/{lang}/work")]
pub async fn work_index(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    query: web::Query<WorkIndexParams>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let status = if query.status.trim().is_empty() { None } else { Some(query.status.trim().to_string()) };
    let unassigned = query.unassigned == "1";
    let page = query.page.trim().parse::<i64>().unwrap_or(1).max(1);
    let offset = (page - 1) * WORK_PAGE_SIZE;

    // The API filters (status + unassigned) and paginates server-side.
    let r = all_work(status, unassigned, Some(WORK_PAGE_SIZE), offset, bearer, &data.api_url, Arc::clone(&data.client)).await;
    let (work, total) = match r {
        Ok(r) => (r.all_work, r.work_count),
        Err(_) => (Vec::new(), 0),
    };

    let total_pages = ((total + WORK_PAGE_SIZE - 1) / WORK_PAGE_SIZE).max(1);

    // Annotate each row with an `overdue` flag for the list badge (Proposal 1).
    let work_items: Vec<serde_json::Value> = work.into_iter().map(|w| {
        let status = domain_key(&w.work_status);
        let overdue = is_overdue(w.due_date, &status);
        let mut v = serde_json::to_value(&w).unwrap_or_else(|_| json!({}));
        v["overdue"] = json!(overdue);
        v
    }).collect();

    ctx.insert("work_items", &work_items);
    ctx.insert("work_statuses", &work_status_options());
    ctx.insert("filter_status", &query.status);
    ctx.insert("filter_unassigned", &unassigned);
    ctx.insert("total", &total);
    ctx.insert("page", &page);
    ctx.insert("total_pages", &total_pages);
    ctx.insert("has_prev", &(page > 1));
    ctx.insert("has_next", &(page < total_pages));

    let rendered = data.tmpl.render("work/work_index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// Vacancy dashboard: vacant roles (needing a person) and vacant work
/// (work items not yet assigned to a role).
#[get("/{lang}/vacancies")]
pub async fn vacancies(
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

    let (roles_res, work_res) = futures::join!(
        vacant_roles(100, bearer.clone(), &data.api_url, Arc::clone(&data.client)),
        // Unassigned work only, filtered server-side (no role assigned).
        all_work(None, true, None, 0, bearer.clone(), &data.api_url, Arc::clone(&data.client)),
    );
    let roles = roles_res.map(|r| r.vacant_roles).unwrap_or_default();

    let vacant_work: Vec<_> = work_res.map(|r| r.all_work).unwrap_or_default();

    ctx.insert("vacant_roles", &roles);
    ctx.insert("vacant_work", &vacant_work);

    let rendered = data.tmpl.render("work/vacancies.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// Proposal 5 — "My Work": a personal worklist aggregating every work item
/// across the signed-in person's active roles, sorted so the most pressing
/// items (overdue, then soonest due, then highest priority) surface first.
#[get("/{lang}/my/work")]
pub async fn my_work_view(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::User) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let person = my_work(auth.bearer, &data.api_url, Arc::clone(&data.client)).await
        .ok()
        .and_then(|r| r.me.person);

    match person {
        Some(p) => {
            // Flatten work across every active role, tagging each item with the
            // role/team it sits under and an overdue flag, then rank it.
            let mut ranked: Vec<(bool, chrono::NaiveDate, i32, serde_json::Value)> = Vec::new();
            for role in &p.active_roles {
                for w in &role.work {
                    let status = domain_key(&w.work_status);
                    let overdue = is_overdue(w.due_date, &status);
                    let due_key = w.due_date.map(|d| d.date()).unwrap_or(chrono::NaiveDate::MAX);
                    let prio_rank = match domain_key(&w.priority).as_str() {
                        "CRITICAL" => 0, "HIGH" => 1, "MEDIUM" => 2, "LOW" => 3, _ => 4,
                    };
                    let mut v = serde_json::to_value(w).unwrap_or_else(|_| json!({}));
                    v["overdue"] = json!(overdue);
                    v["roleId"] = json!(role.id);
                    v["roleTitle"] = json!(role.title_english);
                    v["teamName"] = json!(role.team.name_english);
                    // Overdue first (false > true so negate), then soonest due, then priority.
                    ranked.push((!overdue, due_key, prio_rank, v));
                }
            }
            ranked.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));

            let overdue_count = ranked.iter().filter(|(not_overdue, _, _, _)| !*not_overdue).count() as i64;
            let blocked_count = ranked.iter()
                .filter(|(_, _, _, v)| v["workStatus"] == "BLOCKED").count() as i64;
            let total = ranked.len() as i64;
            let work_items: Vec<serde_json::Value> = ranked.into_iter().map(|(_, _, _, v)| v).collect();

            ctx.insert("person_name", &format!("{} {}", p.given_name, p.family_name));
            ctx.insert("has_person", &true);
            ctx.insert("work_items", &work_items);
            ctx.insert("summary", &json!({
                "total": total,
                "overdue": overdue_count,
                "blocked": blocked_count,
            }));
        },
        None => {
            // Authenticated account with no linked person (e.g. an admin).
            ctx.insert("has_person", &false);
        },
    }

    let rendered = data.tmpl.render("work/my_work.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// Proposal 3 — post a comment or raise a flag on a work item. Open to any
/// authenticated user; the API enforces option (a) (must manage the task or
/// occupy the work's assigned role).
#[post("/{lang}/work/{work_id}/update")]
pub async fn add_work_update_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<WorkUpdateForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, work_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::User) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/work/{}", &lang, &work_id));
    }

    if form.body.trim().is_empty() {
        security::add_flash(&session, "danger", by_lang(&lang, "Please enter a message.", "Veuillez saisir un message."));
        return redirect_to(format!("/{}/work/{}", &lang, &work_id));
    }

    let kind = if form.kind == "FLAG" {
        add_work_update::WorkUpdateKind::FLAG
    } else {
        add_work_update::WorkUpdateKind::COMMENT
    };

    match add_work_update(work_id.clone(), form.body.trim().to_string(), Some(kind), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => {
            security::add_flash(&session, "success", by_lang(&lang, "Update posted.", "Mise à jour publiée."));
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
        },
    }
    redirect_to(format!("/{}/work/{}", &lang, &work_id))
}

/// Proposal 3 — resolve an open flag. Management action (operator+); the API
/// additionally scopes it to the owning task.
#[post("/{lang}/work/{work_id}/flag/{update_id}/resolve")]
pub async fn resolve_work_flag_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String, String)>,
    form: web::Form<CsrfOnlyForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, work_id, update_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/work/{}", &lang, &work_id));
    }

    match resolve_work_update_flag(update_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Flag resolved.", "Signalement résolu.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    }
    // Return to the flags queue when the resolve came from there; otherwise the
    // work page. Only same-site paths are honoured (guard against open redirect).
    if form.return_to.starts_with('/') {
        redirect_to(form.return_to.clone())
    } else {
        redirect_to(format!("/{}/work/{}", &lang, &work_id))
    }
}

/// Manager flags queue (Proposal 3 follow-up): every unresolved flag on work
/// the operator/admin manages, in one place to triage.
#[get("/{lang}/flags")]
pub async fn flags_queue(
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

    let flags = open_work_flags(Some(200), auth.bearer, &data.api_url, Arc::clone(&data.client)).await
        .map(|r| r.open_work_flags)
        .unwrap_or_default();

    ctx.insert("flag_count", &(flags.len() as i64));
    ctx.insert("flags", &flags);

    let rendered = data.tmpl.render("work/flags_queue.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}
