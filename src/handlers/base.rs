use actix_session::{Session, SessionExt};
use actix_web::{web, get, Responder, HttpResponse, HttpRequest};
use actix_identity::Identity;

use std::sync::Arc;
use crate::{generate_basic_context, AppData, graphql::all_organizations};
use crate::security;
use super::utility::{render_page, session_bearer};

#[get("/")]
pub async fn raw_index() -> impl Responder {
    return HttpResponse::Found().header("Location", "/en").finish()
}

// Public, informational page describing the project. No authentication or API
// calls — it only renders static, translated copy.
#[get("/{lang}/about")]
pub async fn about(
    data: web::Data<AppData>,
    params: web::Path<String>,

    id: Option<Identity>,
    req: HttpRequest,
) -> impl Responder {
    let lang = params.into_inner();
    let session = req.get_session();
    let ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    render_page(&data, "about.html", &ctx)
}

// The language segment is constrained to en|fr so that requests like
// /favicon.ico or /robots.txt do not get swallowed by this handler (which
// would clear the session and break the login CSRF token).
#[get("/{lang:en|fr}")]
pub async fn index(
    data: web::Data<AppData>,
    params: web::Path<String>,

    id: Option<Identity>,
    req: HttpRequest,
) -> impl Responder {

    let lang = params.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = session_bearer(&session);

    // Not authenticated: send the visitor to the login page up front instead of
    // firing an API call with an empty token and depending on the API to error.
    if bearer.is_empty() {
        return HttpResponse::Found()
            .append_header(("Location", format!("/{}/log_in", lang)))
            .finish();
    }

    match all_organizations(bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => {
            ctx.insert("organizations", &r.all_organizations);
        }
        Err(e) => {
            let err_msg = format!("{}", e);
            if err_msg.contains("ExpiredSignature")
                || err_msg.contains("InvalidToken")
                || err_msg.contains("Access denied")
                || bearer_is_empty_or_missing(&session)
            {
                // Drop the auth credentials but keep the CSRF token and any
                // flash messages, otherwise the login form we redirect to
                // would render with a token that no longer matches the session.
                clear_auth_keys(&session);
                security::add_flash(
                    &session,
                    "warning",
                    "Your session has expired. Please log in again.",
                );
                return HttpResponse::Found()
                    .append_header(("Location", format!("/{}/log_in", lang)))
                    .finish();
            }
            ctx.insert("organizations", &Vec::<String>::new());
        }
    }

    render_page(&data, "index.html", &ctx)
}

fn bearer_is_empty_or_missing(session: &Session) -> bool {
    match session.get::<String>("bearer") {
        Ok(Some(b)) => b.is_empty(),
        _ => true,
    }
}

/// Remove authentication-related session keys while leaving non-auth state
/// (CSRF token, flash messages) intact.
fn clear_auth_keys(session: &Session) {
    session.remove("bearer");
    session.remove("role");
    session.remove("user_id");
    session.remove("session_user");
    session.remove("expires_at");
}