use actix_session::SessionExt;
use actix_web::{HttpRequest, Responder, get, post, web};
use actix_identity::Identity;
use serde::Deserialize;
use std::sync::Arc;

use crate::{AppData, by_lang, generate_basic_context, security};
use crate::security::MinimumRole;
use crate::graphql::{activate_account, get_me, update_my_person, flag_record_issue};
use super::utility::{redirect_to, csrf_failure_flash, render_page};



// ── Account activation (public) ──────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ActivateQuery {
    pub token: Option<String>,
}

/// Public set-password page reached from an activation link.
#[get("/{lang}/activate")]
pub async fn activate_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    query: web::Query<ActivateQuery>,
    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("token", &query.token.clone().unwrap_or_default());
    render_page(&data, "authentication/activate.html", &ctx)
}

#[derive(Deserialize)]
pub struct ActivateForm {
    pub csrf_token: String,
    pub token: String,
    pub password: String,
    pub password_confirm: String,
}

#[post("/{lang}/activate")]
pub async fn activate_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path: web::Path<String>,
    form: web::Form<ActivateForm>,
    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let back = format!("/{}/activate?token={}", &lang, &form.token);

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(back);
    }
    if form.password != form.password_confirm {
        security::add_flash(&session, "danger", by_lang(&lang, "Passwords do not match.", "Les mots de passe ne correspondent pas."));
        return redirect_to(back);
    }
    if form.password.trim().len() < 8 {
        security::add_flash(&session, "danger", by_lang(&lang, "Password must be at least 8 characters.", "Le mot de passe doit comporter au moins 8 caractères."));
        return redirect_to(back);
    }

    match activate_account(form.token.clone(), form.password.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => {
            security::add_flash(&session, "success", by_lang(&lang, "Account activated. Please sign in.", "Compte activé. Veuillez vous connecter."));
            redirect_to(format!("/{}/log_in", &lang))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            redirect_to(back)
        },
    }
}

// ── Self-service "My profile" ────────────────────────────────────────────────

/// The signed-in person's own profile: view + edit own contact info, flag issues.
#[get("/{lang}/me")]
pub async fn my_profile(
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

    match get_me(auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => ctx.insert("me", &r.me),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    render_page(&data, "account/profile.html", &ctx)
}

#[derive(Deserialize)]
pub struct MyProfileForm {
    pub csrf_token: String,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
    pub phone: String,
    pub work_address: String,
    pub city: String,
    pub province: String,
    pub postal_code: String,
    pub country: String,
}

#[post("/{lang}/me")]
pub async fn my_profile_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path: web::Path<String>,
    form: web::Form<MyProfileForm>,
    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::User) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/me", &lang));
    }

    // Empty fields are left unchanged (None).
    let opt = |s: &str| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) };

    let input = update_my_person::MyPersonUpdate {
        given_name: opt(&form.given_name),
        family_name: opt(&form.family_name),
        email: opt(&form.email),
        phone: opt(&form.phone),
        work_address: opt(&form.work_address),
        city: opt(&form.city),
        province: opt(&form.province),
        postal_code: opt(&form.postal_code),
        country: opt(&form.country),
    };

    match update_my_person(input, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Profile updated.", "Profil mis à jour.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/me", &lang))
}

#[derive(Deserialize)]
pub struct FlagForm {
    pub csrf_token: String,
    pub message: String,
}

#[post("/{lang}/me/flag")]
pub async fn flag_issue_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path: web::Path<String>,
    form: web::Form<FlagForm>,
    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::User) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/me", &lang));
    }
    if form.message.trim().is_empty() {
        security::add_flash(&session, "danger", by_lang(&lang, "Please describe the issue.", "Veuillez décrire le problème."));
        return redirect_to(format!("/{}/me", &lang));
    }

    match flag_record_issue(form.message.trim().to_string(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Thanks — your note was sent to an administrator.", "Merci — votre note a été envoyée à un administrateur.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/me", &lang))
}
