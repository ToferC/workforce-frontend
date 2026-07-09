// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs

use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, HttpMessage, Responder, get, post, web};
use actix_session::{SessionExt};
use actix_identity::{Identity};

use crate::{AppData, by_lang, generate_basic_context, graphql, security};
use crate::graphql::ApiError;

use super::LoginForm;
use super::utility::{render_page};

#[get("/{lang}/log_in")]
pub async fn login_handler(
    path: web::Path<String>,
    data: web::Data<AppData>,
    
    req:HttpRequest,
    id: Option<Identity>,
) -> impl Responder {

    let lang = path.into_inner();

    let session = req.get_session();

    let ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    render_page(&data, "authentication/log_in.html", &ctx)
}

#[post("/{lang}/log_in")]
pub async fn login_form_input(
    path: web::Path<String>,
    data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<LoginForm>,
    _id: Option<Identity>,
) -> impl Responder {

    let lang = path.into_inner();

    let session = req.get_session();

    // Redirect helper: queue a flash message and send the user back to the
    // login form so they see what went wrong instead of a panic / dead end.
    let back_to_login = |level: &str, message: &str| {
        security::add_flash(&session, level, message);
        HttpResponse::Found()
            .append_header(("Location", format!("/{}/log_in", &lang)))
            .finish()
    };

    // Validate CSRF token before processing the login
    if !security::verify_csrf_token(&session, &form.csrf_token) {
        return back_to_login(
            "danger",
            by_lang(
                &lang,
                "Invalid form token. Please try again.",
                "Jeton de formulaire invalide. Veuillez réessayer.",
            ),
        );
    }

    // validate form has data or re-load form
    if form.email.is_empty() || form.password.is_empty() {
        return back_to_login(
            "warning",
            by_lang(
                &lang,
                "Please enter both your email and password.",
                "Veuillez saisir votre courriel et votre mot de passe.",
            ),
        );
    };

    let login_data = match graphql::login(
        form.email.to_lowercase().trim().to_string(),
        form.password.clone(),
        &data.api_url,
        Arc::clone(&data.client),
    )
        .await
    {
        Ok(data) => data.sign_in,
        // The API rejected the credentials. Use a generic message so we don't
        // reveal whether the email is registered (account enumeration).
        Err(ApiError::GraphQL(_)) => {
            return back_to_login(
                "danger",
                by_lang(
                    &lang,
                    "The email or password you entered is incorrect. Please try again.",
                    "Le courriel ou le mot de passe saisi est incorrect. Veuillez réessayer.",
                ),
            );
        }
        // Network failure or empty response from the API — not the user's fault.
        Err(ApiError::Request(_)) | Err(ApiError::MissingData) => {
            return back_to_login(
                "danger",
                by_lang(
                    &lang,
                    "We couldn't sign you in right now. Please try again in a moment.",
                    "Connexion impossible pour le moment. Veuillez réessayer dans un instant.",
                ),
            );
        }
    };

    // Add user_name and role to session
    if Identity::login(&req.extensions(), login_data.email.to_owned()).is_err() {
        return back_to_login(
            "danger",
            by_lang(
                &lang,
                "We couldn't start your session. Please try again.",
                "Impossible de démarrer votre session. Veuillez réessayer.",
            ),
        );
    }

    // The API stores roles in uppercase ("ADMIN"); normalize so template
    // checks like role == "admin" and handler guards compare consistently
    session.insert("role", login_data.role.to_lowercase())
        .expect("Unable to set role");

    session.insert("user_id", login_data.id.to_owned())
        .expect("Unable to set user_id");

    session.insert("session_user", login_data.email.to_owned())
        .expect("Unable to set user name");

    session.insert("bearer", login_data.bearer.to_owned())
        .expect("Unable to set bearer");

    // Store session expiration time as ISO string

    session.insert("expires_at", login_data.expires_at.to_string())
        .expect("Unable to set expires_at");
    

    return HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish()
}

#[get("/{lang}/log_out")]
pub async fn logout(
    path: web::Path<String>,
    _data: web::Data<AppData>,
    req: HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    let lang = path.into_inner();

    let session = req.get_session();

    session.clear();
    // Logging out without a live identity (expired cookie, direct GET) is a
    // no-op, not a panic.
    if let Some(identity) = id {
        identity.logout();
    }

    HttpResponse::Found().append_header(("Location", format!("/{}", &lang))).finish()
}