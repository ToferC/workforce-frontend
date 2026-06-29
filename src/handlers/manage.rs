// The manager panel: a manager's home for the people work that the reporting
// spine enables. v1 surfaces transfer offers — incoming (awaiting my decision)
// and outgoing (offers I've made) — with accept / decline / withdraw actions.
// The API scopes both lists to the signed-in manager.

use actix_session::{Session, SessionExt};
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::Identity;
use serde::Deserialize;
use std::sync::Arc;

use crate::{AppData, by_lang, generate_basic_context};
use crate::graphql::{
    incoming_role_offers, outgoing_role_offers,
    accept_role_offer, decline_role_offer, withdraw_role_offer,
};
use crate::security::{self, MinimumRole};

fn redirect_to(location: String) -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", location))
        .finish()
}

fn csrf_failure_flash(session: &Session, lang: &str) {
    security::add_flash(
        session,
        "danger",
        by_lang(lang, "Invalid form token. Please try again.", "Jeton de formulaire invalide. Veuillez réessayer."),
    );
}

/// A decision on an offer: just a CSRF token and an optional note.
#[derive(Deserialize)]
pub struct OfferDecisionForm {
    pub csrf_token: String,
    pub note: Option<String>,
}

/// Full-page manager panel listing incoming and outgoing transfer offers.
#[get("/{lang}/manage")]
pub async fn manage_panel(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<String>,

    req: HttpRequest) -> impl Responder {
    let lang = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let incoming = match incoming_role_offers(auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.incoming_role_offers,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            Vec::new()
        }
    };

    let outgoing = match outgoing_role_offers(auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.outgoing_role_offers,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            Vec::new()
        }
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("incoming", &incoming);
    ctx.insert("outgoing", &outgoing);

    let rendered = data.tmpl.render("manage/panel.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

fn note_opt(form: &OfferDecisionForm) -> Option<String> {
    form.note.as_ref().map(|n| n.trim().to_string()).filter(|n| !n.is_empty())
}

/// Accept an incoming offer — executes the transfer on the API.
#[post("/{lang}/role_offer/{offer_id}/accept")]
pub async fn accept_offer_post(
    data: web::Data<AppData>,
    path_params: web::Path<(String, String)>,
    form: web::Form<OfferDecisionForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, offer_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };
    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/manage", &lang));
    }

    match accept_role_offer(offer_id, note_opt(&form), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang,
            "Offer accepted — the person has been transferred.",
            "Offre acceptée — la personne a été transférée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    }
    redirect_to(format!("/{}/manage", &lang))
}

/// Decline an incoming offer.
#[post("/{lang}/role_offer/{offer_id}/decline")]
pub async fn decline_offer_post(
    data: web::Data<AppData>,
    path_params: web::Path<(String, String)>,
    form: web::Form<OfferDecisionForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, offer_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };
    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/manage", &lang));
    }

    match decline_role_offer(offer_id, note_opt(&form), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang,
            "Offer declined.", "Offre refusée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    }
    redirect_to(format!("/{}/manage", &lang))
}

/// Withdraw an outgoing offer I made.
#[post("/{lang}/role_offer/{offer_id}/withdraw")]
pub async fn withdraw_offer_post(
    data: web::Data<AppData>,
    path_params: web::Path<(String, String)>,
    form: web::Form<OfferDecisionForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, offer_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };
    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/manage", &lang));
    }

    match withdraw_role_offer(offer_id, note_opt(&form), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang,
            "Offer withdrawn.", "Offre retirée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    }
    redirect_to(format!("/{}/manage", &lang))
}
