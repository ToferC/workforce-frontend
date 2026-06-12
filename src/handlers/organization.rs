use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::{Identity};
use serde::Deserialize;
use serde_json::json;

use std::sync::Arc;
use crate::{AppData, generate_basic_context, by_lang};
use crate::graphql::{get_organization_by_id, create_organization, update_organization};
use crate::security::{self, MinimumRole};

#[derive(Deserialize, Debug)]
pub struct OrganizationForm {
    pub csrf_token: String,
    pub name_en: String,
    pub name_fr: String,
    pub acronym_en: String,
    pub acronym_fr: String,
    pub org_type: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct RetireForm {
    pub csrf_token: String,
}

#[get("/{lang}/organization/{organization_id}")]
pub async fn organization_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req:HttpRequest) -> impl Responder {
    let (lang, organization_id) = path_params.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_organization_by_id(organization_id, bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get organization");

    ctx.insert("organization", &r.organization_by_id);

    let rendered = data.tmpl.render("organization/organization.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// An empty organization object so the form template can always
/// reference the same field names whether creating or editing.
fn blank_organization() -> serde_json::Value {
    json!({
        "nameEn": "",
        "nameFr": "",
        "acronymEn": "",
        "acronymFr": "",
        "orgType": "",
        "url": "",
    })
}

/// Rebuild the template's organization object from a submitted form so the
/// user's input is preserved when re-rendering after an error.
fn organization_from_form(form: &OrganizationForm, id: Option<&str>) -> serde_json::Value {
    json!({
        "id": id,
        "nameEn": form.name_en,
        "nameFr": form.name_fr,
        "acronymEn": form.acronym_en,
        "acronymFr": form.acronym_fr,
        "orgType": form.org_type,
        "url": form.url,
    })
}

fn redirect_to(location: String) -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", location))
        .finish()
}

fn csrf_failure_flash(session: &actix_session::Session, lang: &str) {
    security::add_flash(
        session,
        "danger",
        by_lang(lang, "Invalid form token. Please try again.", "Jeton de formulaire invalide. Veuillez réessayer."),
    );
}

#[get("/{lang}/organization/new")]
pub async fn create_organization_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    if let Err(response) = security::require_role(&session, &lang, MinimumRole::Operator) {
        return response;
    }

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &false);
    ctx.insert("organization", &blank_organization());

    let rendered = data.tmpl.render("organization/organization_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/organization/new")]
pub async fn create_organization_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    form: web::Form<OrganizationForm>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/organization/new", &lang));
    }

    let new_organization = create_organization::NewOrganization {
        name_en: form.name_en.trim().to_string(),
        name_fr: form.name_fr.trim().to_string(),
        acronym_en: form.acronym_en.trim().to_string(),
        acronym_fr: form.acronym_fr.trim().to_string(),
        org_type: form.org_type.trim().to_string(),
        url: form.url.trim().to_string(),
    };

    match create_organization(new_organization, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Organization created.", "Organisation créée."),
            );
            redirect_to(format!("/{}/organization/{}", &lang, response.create_organization.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());

            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &false);
            ctx.insert("organization", &organization_from_form(&form, None));

            let rendered = data.tmpl.render("organization/organization_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}

#[get("/{lang}/organization/{organization_id}/edit")]
pub async fn edit_organization_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, organization_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_organization_by_id(organization_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &true);
    ctx.insert("organization", &r.organization_by_id);

    let rendered = data.tmpl.render("organization/organization_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/organization/{organization_id}/edit")]
pub async fn edit_organization_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<OrganizationForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, organization_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/organization/{}/edit", &lang, &organization_id));
    }

    let organization_data = update_organization::OrganizationData {
        id: organization_id.clone(),
        name_en: Some(form.name_en.trim().to_string()),
        name_fr: Some(form.name_fr.trim().to_string()),
        acronym_en: Some(form.acronym_en.trim().to_string()),
        acronym_fr: Some(form.acronym_fr.trim().to_string()),
        org_type: Some(form.org_type.trim().to_string()),
        url: Some(form.url.trim().to_string()),
        retired_at: None,
    };

    match update_organization(organization_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Organization updated.", "Organisation mise à jour."),
            );
            redirect_to(format!("/{}/organization/{}", &lang, response.update_organization.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());

            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &true);
            ctx.insert("organization", &organization_from_form(&form, Some(&organization_id)));

            let rendered = data.tmpl.render("organization/organization_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}

#[get("/{lang}/organization/{organization_id}/retire")]
pub async fn retire_organization_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, organization_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_organization_by_id(organization_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("organization", &r.organization_by_id);

    let rendered = data.tmpl.render("organization/organization_retire.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/organization/{organization_id}/retire")]
pub async fn retire_organization_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<RetireForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, organization_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/organization/{}", &lang, &organization_id));
    }

    // The API has no delete mutations: retiring sets retired_at on update
    let organization_data = update_organization::OrganizationData {
        id: organization_id.clone(),
        name_en: None,
        name_fr: None,
        acronym_en: None,
        acronym_fr: None,
        org_type: None,
        url: None,
        retired_at: Some(chrono::Utc::now().naive_utc()),
    };

    match update_organization(organization_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => {
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Organization retired.", "Organisation retirée."),
            );
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
        },
    };

    redirect_to(format!("/{}/organization/{}", &lang, &organization_id))
}
