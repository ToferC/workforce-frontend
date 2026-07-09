use actix_session::Session;
use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use tera::Context;

use crate::{AppData, by_lang};
use crate::security;

// ── Shared handler helpers ───────────────────────────────────────────────────
// One definition each for the redirect / CSRF-flash / HTMX-detection / bearer
// plumbing that every entity module needs, instead of a private copy per file.

/// 302 redirect to an app path.
pub fn redirect_to(location: String) -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", location))
        .finish()
}

/// Queue the standard bilingual "invalid form token" flash message.
pub fn csrf_failure_flash(session: &Session, lang: &str) {
    security::add_flash(
        session,
        "danger",
        by_lang(lang, "Invalid form token. Please try again.", "Jeton de formulaire invalide. Veuillez réessayer."),
    );
}

/// True when the request came from HTMX and should get a partial back.
pub fn is_htmx(req: &HttpRequest) -> bool {
    req.headers().get("HX-Request").is_some()
}

/// The session's API bearer token, or "" when there is no live session. Never
/// panics on a corrupt session cookie — the API rejects the empty token.
pub fn session_bearer(session: &Session) -> String {
    session.get::<String>("bearer").ok().flatten().unwrap_or_default()
}

/// Render a template into a 200 response, degrading to a plain 500 if Tera
/// fails. Render errors are programming bugs (the render tests catch missing
/// context), but a panic here would poison the whole worker thread.
pub fn render_page(data: &AppData, template: &str, ctx: &Context) -> HttpResponse {
    match data.tmpl.render(template, ctx) {
        Ok(html) => HttpResponse::Ok().body(html),
        Err(e) => {
            log::error!("Template render failed for {}: {:#}", template, e);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}

/// Flip the language prefix on any app path and redirect there. The header's
/// toggle links to `/toggle_language{current path}`, so this must accept a
/// tail of any depth — fixed-depth routes broke the toggle on deep pages like
/// `/{lang}/role/{id}/requirement/{rid}/edit`.
#[get("/toggle_language/{lang}{tail:.*}")]
pub async fn toggle_language(
    _path: web::Path<(String, String)>,
    req: HttpRequest,
) -> impl Responder {
    // Work from the raw (still percent-encoded) request path so encoded
    // segments round-trip into the redirect untouched.
    let rest = req
        .uri()
        .path()
        .strip_prefix("/toggle_language/")
        .unwrap_or("");
    let (current_lang, tail) = match rest.split_once('/') {
        Some((lang, tail)) => (lang, Some(tail)),
        None => (rest, None),
    };

    let new_lang = match current_lang {
        "en" => "fr",
        _ => "en",
    };

    let location = match tail {
        Some(tail) if !tail.is_empty() => format!("/{}/{}", new_lang, tail),
        _ => format!("/{}", new_lang),
    };

    HttpResponse::Found()
        .append_header(("Location", location))
        .finish()
}
