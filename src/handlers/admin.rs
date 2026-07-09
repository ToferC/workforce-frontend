use actix_session::SessionExt;
use actix_web::{HttpRequest, Responder, get, post, web};
use actix_identity::Identity;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::{AppData, by_lang, generate_basic_context, security};
use crate::security::MinimumRole;
use crate::graphql::{
    all_users, get_user_by_id, create_user, update_user,
    disable_user, enable_user, invite_user, record_flags, resolve_record_flag,
};
use super::utility::{redirect_to, csrf_failure_flash, render_page};



fn role_options() -> serde_json::Value {
    json!([
        {"value": "USER", "label": "User"},
        {"value": "ANALYST", "label": "Analyst"},
        {"value": "OPERATOR", "label": "Operator"},
        {"value": "ADMIN", "label": "Admin"},
    ])
}

fn account_type_options() -> serde_json::Value {
    json!([
        {"value": "HUMAN", "label": "Human"},
        {"value": "AGENT", "label": "Agent (service account)"},
    ])
}

// ── User list ────────────────────────────────────────────────────────────────

#[get("/{lang}/admin/users")]
pub async fn admin_users(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Admin) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    match all_users(auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => ctx.insert("users", &r.all_users),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    render_page(&data, "admin/users.html", &ctx)
}

// ── Create user ──────────────────────────────────────────────────────────────

#[get("/{lang}/admin/users/new")]
pub async fn admin_user_new_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    if let Err(response) = security::require_role(&session, &lang, MinimumRole::Admin) {
        return response;
    }

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &false);
    ctx.insert("user", &json!({"id": "", "name": "", "email": "", "role": "USER", "accountType": "HUMAN"}));
    ctx.insert("role_options", &role_options());
    ctx.insert("account_type_options", &account_type_options());

    render_page(&data, "admin/user_form.html", &ctx)
}

#[derive(Deserialize, Debug)]
pub struct UserCreateForm {
    pub csrf_token: String,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub account_type: String,
}

#[post("/{lang}/admin/users/new")]
pub async fn admin_user_create(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path: web::Path<String>,
    form: web::Form<UserCreateForm>,
    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Admin) {
        Ok(auth) => auth,
        Err(response) => return response,
    };
    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/admin/users/new", &lang));
    }

    let user_data = create_user::UserData {
        name: form.name.trim().to_string(),
        email: form.email.to_lowercase().trim().to_string(),
        password: form.password.clone(),
        role: form.role.clone(),
        account_type: Some(form.account_type.clone()),
    };

    match create_user(user_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => {
            security::add_flash(&session, "success", by_lang(&lang, "User created.", "Utilisateur créé."));
            redirect_to(format!("/{}/admin/users", &lang))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            redirect_to(format!("/{}/admin/users/new", &lang))
        },
    }
}

// ── Edit user ────────────────────────────────────────────────────────────────

#[get("/{lang}/admin/users/{user_id}/edit")]
pub async fn admin_user_edit_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    req: HttpRequest) -> impl Responder {
    let (lang, user_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Admin) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_user_by_id(user_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/admin/users", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &true);
    ctx.insert("user", &r.user_by_id);
    ctx.insert("role_options", &role_options());
    ctx.insert("account_type_options", &account_type_options());

    render_page(&data, "admin/user_form.html", &ctx)
}

#[derive(Deserialize, Debug)]
pub struct UserEditForm {
    pub csrf_token: String,
    pub name: String,
    pub email: String,
    /// Optional — leave blank to keep the current password.
    pub password: String,
    pub role: String,
}

#[post("/{lang}/admin/users/{user_id}/edit")]
pub async fn admin_user_update(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<UserEditForm>,
    req: HttpRequest) -> impl Responder {
    let (lang, user_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Admin) {
        Ok(auth) => auth,
        Err(response) => return response,
    };
    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/admin/users/{}/edit", &lang, &user_id));
    }

    let opt = |s: &str| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) };

    let user_data = update_user::UserUpdate {
        id: user_id.clone(),
        name: opt(&form.name),
        email: opt(&form.email.to_lowercase()),
        password: opt(&form.password),
        role: opt(&form.role),
    };

    match update_user(user_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "User updated.", "Utilisateur mis à jour.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };
    redirect_to(format!("/{}/admin/users", &lang))
}

// ── Lifecycle actions ────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
pub struct CsrfOnlyForm {
    pub csrf_token: String,
}

#[post("/{lang}/admin/users/{user_id}/invite")]
pub async fn admin_user_invite(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<CsrfOnlyForm>,
    req: HttpRequest) -> impl Responder {
    let (lang, user_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Admin) {
        Ok(auth) => auth,
        Err(response) => return response,
    };
    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/admin/users", &lang));
    }

    match invite_user(user_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(resp) => {
            let link = format!("/{}/activate?token={}", &lang, resp.invite_user.activation_token);
            security::add_flash(&session, "success", &by_lang(
                &lang,
                &format!("Invitation issued. Activation link: {}", link),
                &format!("Invitation émise. Lien d'activation : {}", link),
            ));
        },
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };
    redirect_to(format!("/{}/admin/users", &lang))
}

#[post("/{lang}/admin/users/{user_id}/disable")]
pub async fn admin_user_disable(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<CsrfOnlyForm>,
    req: HttpRequest) -> impl Responder {
    let (lang, user_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Admin) {
        Ok(auth) => auth,
        Err(response) => return response,
    };
    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/admin/users", &lang));
    }

    match disable_user(user_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "User disabled.", "Utilisateur désactivé.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };
    redirect_to(format!("/{}/admin/users", &lang))
}

#[post("/{lang}/admin/users/{user_id}/enable")]
pub async fn admin_user_enable(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<CsrfOnlyForm>,
    req: HttpRequest) -> impl Responder {
    let (lang, user_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Admin) {
        Ok(auth) => auth,
        Err(response) => return response,
    };
    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/admin/users", &lang));
    }

    match enable_user(user_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "User enabled.", "Utilisateur activé.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };
    redirect_to(format!("/{}/admin/users", &lang))
}

// ── Record flags review queue ────────────────────────────────────────────────

#[get("/{lang}/admin/flags")]
pub async fn admin_flags(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Admin) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    match record_flags(auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => ctx.insert("flags", &r.record_flags),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    render_page(&data, "admin/flags.html", &ctx)
}

#[post("/{lang}/admin/flags/{flag_id}/resolve")]
pub async fn admin_flag_resolve(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<CsrfOnlyForm>,
    req: HttpRequest) -> impl Responder {
    let (lang, flag_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Admin) {
        Ok(auth) => auth,
        Err(response) => return response,
    };
    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/admin/flags", &lang));
    }

    match resolve_record_flag(flag_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Flag resolved.", "Signalement résolu.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };
    redirect_to(format!("/{}/admin/flags", &lang))
}
