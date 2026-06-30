//! Transactional notifications for the transfer-offer workflow.
//!
//! Deliberately **off by default** for the MVP: nothing is sent unless
//! `OFFER_NOTIFICATIONS_ENABLED` is set truthy *and* a SendGrid key is present.
//! When disabled, `send_offer_email` is a no-op, so the offer handlers behave
//! exactly as if email didn't exist. Flip the flag (and set `SENDGRID_API_KEY`)
//! to activate — no code change needed.

use sendgrid::SGClient;

use crate::models::Email;

/// Whether offer notification emails are sent. Off unless
/// `OFFER_NOTIFICATIONS_ENABLED` is `true`/`1`.
pub fn offer_notifications_enabled() -> bool {
    matches!(
        std::env::var("OFFER_NOTIFICATIONS_ENABLED").ok().as_deref(),
        Some("true") | Some("1")
    )
}

/// Best-effort send of one offer notification. No-op when notifications are
/// disabled or no recipient/key is available; a send failure is logged but never
/// surfaced to the caller, so a mail problem can't break the offer action.
pub async fn send_offer_email(to: Option<&str>, subject: &str, html: &str) {
    if !offer_notifications_enabled() {
        return;
    }

    let to = match to {
        Some(addr) if !addr.trim().is_empty() => addr,
        _ => return, // no recipient resolved; nothing to do
    };

    let key = match std::env::var("SENDGRID_API_KEY") {
        Ok(k) if !k.trim().is_empty() => k,
        _ => {
            eprintln!("offer notifications enabled but SENDGRID_API_KEY is not set; skipping");
            return;
        }
    };

    let email = Email::new(to.to_string(), html.to_string(), subject.to_string(), SGClient::new(key));
    if let Err(e) = Email::send(&email).await {
        eprintln!("offer notification email to {to} failed: {e:?}");
    }
}
