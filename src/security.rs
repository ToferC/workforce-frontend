use actix_session::Session;
use actix_web::HttpResponse;
use chrono::NaiveDateTime;
use rand::Rng;
use rand::distributions::Alphanumeric;
use serde::{Deserialize, Serialize};

use crate::extract_session_data;

const CSRF_SESSION_KEY: &str = "csrf_token";
const FLASH_SESSION_KEY: &str = "flash_messages";

/// Minimum role required to access a handler. Mirrors the API's
/// UserRole hierarchy: user < analyst < operator < admin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MinimumRole {
    User = 1,
    Analyst = 2,
    Operator = 3,
    Admin = 4,
}

fn role_rank(role: &str) -> u8 {
    match role.to_lowercase().as_str() {
        "admin" => 4,
        "operator" => 3,
        "analyst" => 2,
        "user" => 1,
        _ => 0,
    }
}

/// Session data needed by handlers that call guarded API mutations.
#[derive(Debug, Clone)]
pub struct AuthorizedSession {
    pub bearer: String,
    pub role: String,
    pub user_id: String,
}

/// Enforce authentication and a minimum role before running a handler.
///
/// Returns the bearer token and user info on success. On failure returns
/// a redirect response the handler should return immediately:
/// to the log-in page when there is no live session, or to the
/// not-authorized page when the user's role is insufficient.
///
/// ```ignore
/// let auth = match require_role(&session, &lang, MinimumRole::Operator) {
///     Ok(auth) => auth,
///     Err(response) => return response,
/// };
/// ```
pub fn require_role(
    session: &Session,
    lang: &str,
    minimum: MinimumRole,
) -> Result<AuthorizedSession, HttpResponse> {
    let bearer = match session.get::<String>("bearer") {
        Ok(Some(b)) if !b.is_empty() => b,
        _ => return Err(redirect_to_login(lang)),
    };

    let (role, user_id, expires_at) = extract_session_data(session);

    if session_expired(&expires_at) {
        session.clear();
        return Err(redirect_to_login(lang));
    }

    if role_rank(&role) < minimum as u8 {
        return Err(HttpResponse::Found()
            .append_header(("Location", format!("/{}/not_authorized", lang)))
            .finish());
    }

    Ok(AuthorizedSession { bearer, role, user_id })
}

fn redirect_to_login(lang: &str) -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", format!("/{}/log_in", lang)))
        .finish()
}

/// The session stores expires_at as the API's NaiveDateTime rendered with
/// to_string(). An unparseable or missing value counts as expired so the
/// user is sent back through log-in rather than calling the API with a
/// token that will be rejected.
fn session_expired(expires_at: &str) -> bool {
    let parsed = NaiveDateTime::parse_from_str(expires_at, "%Y-%m-%d %H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(expires_at, "%Y-%m-%dT%H:%M:%S%.f"));

    match parsed {
        // The API issues expiry in its local clock, so compare local-to-local
        Ok(expiry) => expiry <= chrono::Local::now().naive_local(),
        Err(_) => true,
    }
}

/// Get the session's CSRF token, creating one if needed. Called for every
/// page render (see generate_basic_context) so forms can embed it as a
/// hidden `csrf_token` field.
pub fn get_or_create_csrf_token(session: &Session) -> String {
    if let Ok(Some(token)) = session.get::<String>(CSRF_SESSION_KEY) {
        if !token.is_empty() {
            return token;
        }
    }

    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    session
        .insert(CSRF_SESSION_KEY, token.clone())
        .expect("Unable to store CSRF token in session");

    token
}

/// Verify a submitted form's CSRF token against the session. Every POST
/// handler that mutates data must call this before acting on the form.
pub fn verify_csrf_token(session: &Session, submitted: &str) -> bool {
    match session.get::<String>(CSRF_SESSION_KEY) {
        Ok(Some(stored)) => !stored.is_empty() && stored == submitted,
        _ => false,
    }
}

/// A one-time message shown to the user on the next rendered page.
/// `level` is a Bootstrap alert level: success, danger, warning, info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashMessage {
    pub level: String,
    pub message: String,
}

/// Queue a flash message for the next rendered page.
pub fn add_flash(session: &Session, level: &str, message: &str) {
    let mut messages = match session.get::<Vec<FlashMessage>>(FLASH_SESSION_KEY) {
        Ok(Some(m)) => m,
        _ => Vec::new(),
    };

    messages.push(FlashMessage {
        level: level.to_string(),
        message: message.to_string(),
    });

    session
        .insert(FLASH_SESSION_KEY, messages)
        .expect("Unable to store flash messages in session");
}

/// Take and clear queued flash messages. Called by generate_basic_context
/// so messages render once and then disappear.
pub fn take_flash(session: &Session) -> Vec<FlashMessage> {
    let messages = match session.get::<Vec<FlashMessage>>(FLASH_SESSION_KEY) {
        Ok(Some(m)) => m,
        _ => Vec::new(),
    };

    if !messages.is_empty() {
        session.remove(FLASH_SESSION_KEY);
    }

    messages
}
