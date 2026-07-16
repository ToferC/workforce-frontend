use actix_session::SessionExt;
use actix_web::{HttpRequest, Responder, get, post, web};
use actix_identity::{Identity};
use serde::Deserialize;
use serde_json::json;

use std::sync::Arc;
use crate::{AppData, generate_basic_context, by_lang};
use crate::graphql::{get_organization_by_id, all_organizations, create_organization, update_organization, restore_organization, org_tier_financials};
use crate::security::{self, MinimumRole};
use super::utility::{redirect_to, csrf_failure_flash, is_htmx, render_page, session_bearer};


#[derive(Deserialize, Debug)]
pub struct OrgIndexParams {
    #[serde(default)]
    pub retired: String,
    #[serde(default)]
    pub q: String,
}

/// Index of all organizations, with name/acronym search and an optional
/// retired filter. Mirrors the team/person index pattern (HTMX live search
/// re-renders just the list partial).
#[get("/{lang}/organizations")]
pub async fn organization_index(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    params: web::Query<OrgIndexParams>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = session_bearer(&req.get_session());

    let show_retired = params.retired == "1";
    let query = params.q.trim().to_lowercase();
    let orgs = all_organizations(bearer, &data.api_url, Arc::clone(&data.client)).await
        .map(|r| r.all_organizations)
        .unwrap_or_default();

    let matched: Vec<_> = orgs.iter()
        .filter(|o| show_retired || o.retired_at.is_none())
        .filter(|o| query.is_empty()
            || o.name_en.to_lowercase().contains(&query)
            || o.name_fr.to_lowercase().contains(&query)
            || o.acronym_en.to_lowercase().contains(&query)
            || o.acronym_fr.to_lowercase().contains(&query))
        .collect();
    let total = matched.len();
    let visible: Vec<_> = matched.into_iter().take(super::person::INDEX_PAGE_CAP).collect();

    ctx.insert("organizations", &visible);
    ctx.insert("total", &total);
    ctx.insert("truncated", &(total > super::person::INDEX_PAGE_CAP));
    ctx.insert("q", &params.q);
    ctx.insert("show_retired", &show_retired);

    let template = if is_htmx(&req) { "organization/organization_list.html" } else { "organization/organization_index.html" };
    render_page(&data, template, &ctx)
}

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

    let bearer = session_bearer(&req.get_session());

    let r = match get_organization_by_id(organization_id, bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/organizations", &lang));
        },
    };

    ctx.insert("organization", &r.organization_by_id);

    // At-a-glance tiles: tier count and fiscal-year money for the org,
    // summed over its top tiers' subtrees. Best-effort — the page still
    // renders without it.
    let mut tier_count = 0usize;
    let (mut budgeted, mut projected, mut lapse, mut allocation) = (0i64, 0i64, 0i64, 0i64);
    let mut have_finances = false;
    for root in &r.organization_by_id.top_org_tier {
        if let Ok(fin) = org_tier_financials(9, Some(root.id.clone()), None, bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
            let rows = fin.org_tier_financials;
            tier_count += rows.len();
            if let Some(own) = rows.iter().find(|row| row.org_tier_id == root.id) {
                budgeted += own.budgeted_cents;
                projected += own.projected_cents;
                lapse += own.lapse_cents;
                allocation += own.allocation_cents.unwrap_or(0);
                have_finances = true;
            }
        }
    }
    if have_finances {
        ctx.insert("org_finances", &serde_json::json!({
            "tiers": tier_count,
            "budgetedCents": budgeted,
            "projectedCents": projected,
            "lapseCents": lapse,
            "allocationCents": allocation,
        }));
    }

    render_page(&data, "organization/organization.html", &ctx)
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

    render_page(&data, "organization/organization_form.html", &ctx)
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
            // The API seeds a starter Executive tier with a vacant "Head" role
            // (see createOrganization). Point the operator straight at the org
            // chart so they can staff that role instead of landing on a
            // read-only detail page.
            security::add_flash(
                &session,
                "success",
                by_lang(
                    &lang,
                    "Organization created with a starter Executive tier. Assign a person to its vacant Head role to staff it.",
                    "Organisation créée avec un niveau exécutif de départ. Assignez une personne à son rôle de responsable vacant pour la doter.",
                ),
            );
            redirect_to(format!("/{}/organization/{}/org_chart", &lang, response.create_organization.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());

            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &false);
            ctx.insert("organization", &organization_from_form(&form, None));

            render_page(&data, "organization/organization_form.html", &ctx)
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

    render_page(&data, "organization/organization_form.html", &ctx)
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

            render_page(&data, "organization/organization_form.html", &ctx)
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

    render_page(&data, "organization/organization_retire.html", &ctx)
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

#[post("/{lang}/organization/{organization_id}/restore")]
pub async fn restore_organization_post(
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

    match restore_organization(organization_id.clone(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Organization restored.", "Organisation restaurée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/organization/{}", &lang, &organization_id))
}
