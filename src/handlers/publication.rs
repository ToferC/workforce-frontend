use actix_session::SessionExt;
use actix_web::{HttpRequest, Responder, get, post, web};
use actix_identity::{Identity};
use serde::Deserialize;
use serde_json::json;

use std::sync::Arc;
use crate::{AppData, generate_basic_context, by_lang};
use crate::graphql::{get_publication_by_id, all_publications, create_publication, update_publication};
use crate::security::{self, MinimumRole};
use super::org_tier::humanize;
use super::person::{organization_options, resolve_person_by_name};
use super::task::parse_date;
use super::utility::{redirect_to, csrf_failure_flash, render_page, session_bearer};

/// PublicationStatus enum values, kept in sync with the API schema.
pub const PUBLICATION_STATUSES: [&str; 7] = ["PLANNING", "IN_PROGRESS", "DRAFT", "SUBMITTED", "PUBLISHED", "REJECTED", "CANCELLED"];

fn publication_status_options() -> serde_json::Value {
    json!(PUBLICATION_STATUSES.iter().map(|s| json!({"value": s, "label": humanize(s)})).collect::<Vec<serde_json::Value>>())
}



#[derive(Deserialize, Debug)]
pub struct PublicationForm {
    pub csrf_token: String,
    // Create only (lead author & org are immutable in the API after creation)
    #[serde(default)]
    pub organization_id: String,
    #[serde(default)]
    pub lead_author_name: String,
    pub title: String,
    pub subject_text: String,
    pub publication_status: String,
    #[serde(default)]
    pub url_string: String,
    #[serde(default)]
    pub publishing_id: String,
    #[serde(default)]
    pub published_datestamp: String,
}

fn publication_from_form(form: &PublicationForm, id: Option<&str>) -> serde_json::Value {
    json!({
        "id": id,
        "title": form.title,
        "subjectText": form.subject_text,
        "publicationStatus": form.publication_status,
        "urlString": form.url_string,
        "publishingId": form.publishing_id,
        "publishedDatestamp": form.published_datestamp,
        "publishingOrganization": {"id": form.organization_id},
    })
}

#[get("/{lang}/publications")]
pub async fn publication_index(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = session_bearer(&req.get_session());

    // Degrade to an empty list (with the error flashed) if the API is down
    let publications = match all_publications(bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.all_publications,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            Vec::new()
        },
    };
    ctx.insert("publications", &publications);

    render_page(&data, "publication/publication_index.html", &ctx)
}

#[get("/{lang}/publication/{publication_id}")]
pub async fn publication_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req:HttpRequest) -> impl Responder {
    let (lang, publication_id) = path_params.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = session_bearer(&req.get_session());

    let r = match get_publication_by_id(publication_id, bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/publications", &lang));
        },
    };

    ctx.insert("publication", &r.publication_by_id);

    render_page(&data, "publication/publication.html", &ctx)
}

#[get("/{lang}/publication/new")]
pub async fn create_publication_form(
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
    ctx.insert("edit", &false);
    ctx.insert("publication", &json!({
        "title": "", "subjectText": "", "publicationStatus": "PLANNING", "urlString": "",
        "publishingId": "", "publishedDatestamp": "", "publishingOrganization": {"id": ""},
    }));
    ctx.insert("organization_options", &organization_options(&auth.bearer, &data).await);
    ctx.insert("publication_statuses", &publication_status_options());

    render_page(&data, "publication/publication_form.html", &ctx)
}

#[post("/{lang}/publication/new")]
pub async fn create_publication_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    form: web::Form<PublicationForm>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/publication/new", &lang));
    }

    let render_error = |message: String, id: Option<Identity>, options: serde_json::Value| {
        security::add_flash(&session, "danger", &message);
        let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
        ctx.insert("edit", &false);
        ctx.insert("publication", &publication_from_form(&form, None));
        ctx.insert("organization_options", &options);
        ctx.insert("publication_statuses", &publication_status_options());
        render_page(&data, "publication/publication_form.html", &ctx)
    };

    let lead_author_id = match resolve_person_by_name(&form.lead_author_name, &auth.bearer, &lang, &data).await {
        Ok(Some(person_id)) => person_id,
        Ok(None) => {
            let options = organization_options(&auth.bearer, &data).await;
            return render_error(by_lang(&lang, "Enter the lead author's name.", "Entrez le nom de l'auteur principal.").to_string(), id, options);
        },
        Err(message) => {
            let options = organization_options(&auth.bearer, &data).await;
            return render_error(message, id, options);
        },
    };

    let new_publication = create_publication::NewPublication {
        publishing_organization_id: form.organization_id.clone(),
        lead_author_id,
        title: form.title.trim().to_string(),
        subject_text: form.subject_text.trim().to_string(),
        publication_status: serde_json::from_value(json!(form.publication_status)).expect("PublicationStatus deserialization is infallible"),
        url_string: if form.url_string.trim().is_empty() { None } else { Some(form.url_string.trim().to_string()) },
        publishing_id: if form.publishing_id.trim().is_empty() { None } else { Some(form.publishing_id.trim().to_string()) },
        submitted_date: None,
        published_datestamp: parse_date(&form.published_datestamp),
    };

    match create_publication(new_publication, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(&session, "success", by_lang(&lang, "Publication created.", "Publication créée."));
            redirect_to(format!("/{}/publication/{}", &lang, response.create_publication.id))
        },
        Err(e) => {
            let options = organization_options(&auth.bearer, &data).await;
            render_error(e.to_string(), id, options)
        },
    }
}

#[get("/{lang}/publication/{publication_id}/edit")]
pub async fn edit_publication_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, publication_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_publication_by_id(publication_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/publications", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &true);
    ctx.insert("publication", &r.publication_by_id);
    ctx.insert("publication_statuses", &publication_status_options());

    render_page(&data, "publication/publication_form.html", &ctx)
}

#[post("/{lang}/publication/{publication_id}/edit")]
pub async fn edit_publication_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<PublicationForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, publication_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/publication/{}/edit", &lang, &publication_id));
    }

    let publication_data = update_publication::PublicationData {
        id: publication_id.clone(),
        title: Some(form.title.trim().to_string()),
        subject_text: Some(form.subject_text.trim().to_string()),
        publication_status: Some(serde_json::from_value(json!(form.publication_status)).expect("PublicationStatus deserialization is infallible")),
        url_string: if form.url_string.trim().is_empty() { None } else { Some(form.url_string.trim().to_string()) },
        publishing_id: if form.publishing_id.trim().is_empty() { None } else { Some(form.publishing_id.trim().to_string()) },
        submitted_date: None,
        published_datestamp: parse_date(&form.published_datestamp),
    };

    match update_publication(publication_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(&session, "success", by_lang(&lang, "Publication updated.", "Publication mise à jour."));
            redirect_to(format!("/{}/publication/{}", &lang, response.update_publication.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &true);
            ctx.insert("publication", &publication_from_form(&form, Some(&publication_id)));
            ctx.insert("publication_statuses", &publication_status_options());
            render_page(&data, "publication/publication_form.html", &ctx)
        },
    }
}
