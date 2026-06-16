use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::{Identity};
use serde::Deserialize;
use serde_json::json;

use std::sync::Arc;
use crate::{AppData, generate_basic_context, by_lang};
use crate::graphql::{all_org_tiers, get_org_tier_by_id, get_org_tiers_by_org_id, create_org_tier, update_org_tier, create_org_ownership, get_org_ownership_by_tier_id, update_org_ownership, restore_org_tier};
use crate::security::{self, MinimumRole};
use super::person::resolve_person_by_name;

/// SkillDomain enum values, kept in sync with the API schema. Used to
/// populate the primary domain select on tier forms.
pub const SKILL_DOMAINS: [&str; 16] = [
    "COMBAT", "INTELLIGENCE", "STRATEGY", "ENGINEERING", "MEDICAL",
    "JOINT_OPERATIONS", "SOFTWARE_ENGINEERING", "CLOUD_PLATFORM_DEV_OPS",
    "DATA_ANALYTICS_AND_AI", "CYBER_SECURITY", "PRODUCT_AGILE_AND_DELIVERY",
    "USER_EXPERIENCE", "PROCUREMENT_AND_VENDOR_MANAGEMENT",
    "PEOPLE_AND_ORGANISATIONAL_LEADERSHIP", "GOVERNANCE", "CORPORATE_SERVICES",
];

/// Render "INFORMATION_TECHNOLOGY" as "Information Technology" for labels
pub fn humanize(value: &str) -> String {
    value
        .split('_')
        .map(|word| {
            let lower = word.to_lowercase();
            let mut chars = lower.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn skill_domain_options() -> serde_json::Value {
    json!(SKILL_DOMAINS
        .iter()
        .map(|domain| json!({"value": domain, "label": humanize(domain)}))
        .collect::<Vec<serde_json::Value>>())
}

#[derive(Deserialize, Debug)]
pub struct OrgTierForm {
    pub csrf_token: String,
    pub organization_id: String,
    pub name_en: String,
    pub name_fr: String,
    pub tier_level: i64,
    pub primary_domain: String,
    #[serde(default)]
    pub parent_tier: String,
}

#[derive(Deserialize, Debug)]
pub struct RetireForm {
    pub csrf_token: String,
}

#[derive(Deserialize, Debug)]
pub struct NewTierParams {
    pub organization: String,
    #[serde(default)]
    pub parent: String,
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

fn is_htmx(req: &HttpRequest) -> bool {
    req.headers().get("HX-Request").is_some()
}

/// Options for a tier select: every tier of the organization, optionally
/// excluding one (a tier cannot be its own parent). Also used by the
/// team form's org-tier select.
pub async fn parent_tier_options(
    organization_id: &str,
    exclude_tier_id: Option<&str>,
    bearer: &str,
    data: &AppData,
) -> Result<serde_json::Value, crate::graphql::ApiError> {
    let tiers = get_org_tiers_by_org_id(
        organization_id.to_string(),
        bearer.to_string(),
        &data.api_url,
        Arc::clone(&data.client),
    ).await?;

    let options: Vec<serde_json::Value> = tiers.org_tiers_by_org_id
        .iter()
        .filter(|tier| Some(tier.id.as_str()) != exclude_tier_id)
        .map(|tier| json!({
            "value": tier.id,
            "label": format!("{} (level {})", tier.name_en, tier.tier_level),
        }))
        .collect();

    Ok(json!(options))
}

fn org_tier_from_form(form: &OrgTierForm, id: Option<&str>) -> serde_json::Value {
    json!({
        "id": id,
        "nameEn": form.name_en,
        "nameFr": form.name_fr,
        "tierLevel": form.tier_level,
        "primaryDomain": form.primary_domain,
        "organization": {"id": form.organization_id},
        "parentOrganizationTier": if form.parent_tier.is_empty() {
            json!(null)
        } else {
            json!({"id": form.parent_tier})
        },
    })
}

#[derive(Deserialize, Debug)]
pub struct OrgTierIndexParams {
    #[serde(default)]
    pub retired: String,
    #[serde(default)]
    pub q: String,
}

#[get("/{lang}/org_tiers")]
pub async fn org_tier_index(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    params: web::Query<OrgTierIndexParams>,
    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let show_retired = params.retired == "1";
    let query = params.q.trim().to_lowercase();
    let tiers = all_org_tiers(bearer, &data.api_url, Arc::clone(&data.client)).await
        .map(|r| r.all_org_tiers)
        .unwrap_or_default();

    let matched: Vec<_> = tiers.iter()
        .filter(|t| show_retired || t.retired_at.is_none())
        .filter(|t| query.is_empty()
            || t.name_en.to_lowercase().contains(&query)
            || t.name_fr.to_lowercase().contains(&query)
            || t.organization.name_en.to_lowercase().contains(&query)
            || t.organization.name_fr.to_lowercase().contains(&query)
            || t.organization.acronym_en.to_lowercase().contains(&query)
            || t.organization.acronym_fr.to_lowercase().contains(&query))
        .collect();
    let total = matched.len();
    let visible: Vec<_> = matched.into_iter().take(super::person::INDEX_PAGE_CAP).collect();

    ctx.insert("org_tiers", &visible);
    ctx.insert("total", &total);
    ctx.insert("truncated", &(total > super::person::INDEX_PAGE_CAP));
    ctx.insert("q", &params.q);
    ctx.insert("show_retired", &show_retired);

    let template = if is_htmx(&req) { "org_tier/org_tier_list.html" } else { "org_tier/org_tier_index.html" };
    let rendered = data.tmpl.render(template, &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/org_tier/{org_tier_id}")]
pub async fn org_tier_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req:HttpRequest) -> impl Responder {
    let (lang, org_tier_id) = path_params.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_org_tier_by_id(org_tier_id, bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get org tier");

    ctx.insert("org_tier", &r.org_tier_by_id);

    let rendered = data.tmpl.render("org_tier/org_tier.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// Form to create an org tier. Takes ?organization=<uuid> and optionally
/// &parent=<uuid> so the builder can link here with the parent pre-selected.
/// When requested by HTMX, renders only the inline form partial.
#[get("/{lang}/org_tier/new")]
pub async fn create_org_tier_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    params: web::Query<NewTierParams>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let parent_options = match parent_tier_options(&params.organization, None, &auth.bearer, &data).await {
        Ok(options) => options,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/organization/{}", &lang, &params.organization));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &false);
    ctx.insert("org_tier", &json!({
        "nameEn": "", "nameFr": "", "tierLevel": 1, "primaryDomain": "",
        "organization": {"id": params.organization},
        "parentOrganizationTier": if params.parent.is_empty() { json!(null) } else { json!({"id": params.parent}) },
    }));
    ctx.insert("parent_tier_options", &parent_options);
    ctx.insert("skill_domains", &skill_domain_options());

    let template = if is_htmx(&req) {
        "org_chart/add_tier_form.html"
    } else {
        "org_tier/org_tier_form.html"
    };

    let rendered = data.tmpl.render(template, &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/org_tier/new")]
pub async fn create_org_tier_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    form: web::Form<OrgTierForm>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/org_tier/new?organization={}", &lang, &form.organization_id));
    }

    let new_tier = create_org_tier::NewOrgTier {
        organization_id: form.organization_id.clone(),
        tier_level: form.tier_level,
        name_en: form.name_en.trim().to_string(),
        name_fr: form.name_fr.trim().to_string(),
        // The generated enum deserializes unknown values to Other(..),
        // which the API rejects with a clear error, so this can't panic
        primary_domain: serde_json::from_value(json!(form.primary_domain))
            .expect("SkillDomain deserialization is infallible"),
        parent_tier: if form.parent_tier.is_empty() { None } else { Some(form.parent_tier.clone()) },
    };

    match create_org_tier(new_tier, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            if is_htmx(&req) {
                // In the builder: re-render the parent node (or tell the
                // chart to refresh from the top when a root tier was added)
                return super::org_chart::render_node_response(
                    &data, &session, id, &lang, &form.parent_tier, &form.organization_id, &req,
                ).await;
            }
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Organization tier created.", "Niveau organisationnel créé."),
            );
            redirect_to(format!("/{}/org_tier/{}", &lang, response.create_org_tier.id))
        },
        Err(e) => {
            // Flash renders only on full pages; the inline partial shows
            // the error itself via form_error
            if !is_htmx(&req) {
                security::add_flash(&session, "danger", &e.to_string());
            }

            let parent_options = parent_tier_options(&form.organization_id, None, &auth.bearer, &data)
                .await
                .unwrap_or_else(|_| json!([]));

            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &false);
            ctx.insert("org_tier", &org_tier_from_form(&form, None));
            ctx.insert("parent_tier_options", &parent_options);
            ctx.insert("skill_domains", &skill_domain_options());

            let template = if is_htmx(&req) {
                ctx.insert("form_error", &e.to_string());
                "org_chart/add_tier_form.html"
            } else {
                "org_tier/org_tier_form.html"
            };

            let rendered = data.tmpl.render(template, &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}

#[get("/{lang}/org_tier/{org_tier_id}/edit")]
pub async fn edit_org_tier_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, org_tier_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_org_tier_by_id(org_tier_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let parent_options = parent_tier_options(
        &r.org_tier_by_id.organization.id,
        Some(&org_tier_id),
        &auth.bearer,
        &data,
    ).await.unwrap_or_else(|_| json!([]));

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &true);
    ctx.insert("org_tier", &r.org_tier_by_id);
    ctx.insert("parent_tier_options", &parent_options);
    ctx.insert("skill_domains", &skill_domain_options());

    let rendered = data.tmpl.render("org_tier/org_tier_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/org_tier/{org_tier_id}/edit")]
pub async fn edit_org_tier_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<OrgTierForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, org_tier_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/org_tier/{}/edit", &lang, &org_tier_id));
    }

    let tier_data = update_org_tier::OrgTierData {
        id: org_tier_id.clone(),
        name_en: Some(form.name_en.trim().to_string()),
        name_fr: Some(form.name_fr.trim().to_string()),
        tier_level: Some(form.tier_level),
        primary_domain: Some(serde_json::from_value(json!(form.primary_domain))
            .expect("SkillDomain deserialization is infallible")),
        parent_tier: if form.parent_tier.is_empty() { None } else { Some(form.parent_tier.clone()) },
        retired_at: None,
    };

    match update_org_tier(tier_data, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Organization tier updated.", "Niveau organisationnel mis à jour."),
            );
            redirect_to(format!("/{}/org_tier/{}", &lang, response.update_org_tier.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());

            let parent_options = parent_tier_options(&form.organization_id, Some(&org_tier_id), &auth.bearer, &data)
                .await
                .unwrap_or_else(|_| json!([]));

            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &true);
            ctx.insert("org_tier", &org_tier_from_form(&form, Some(&org_tier_id)));
            ctx.insert("parent_tier_options", &parent_options);
            ctx.insert("skill_domains", &skill_domain_options());

            let rendered = data.tmpl.render("org_tier/org_tier_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}

#[get("/{lang}/org_tier/{org_tier_id}/retire")]
pub async fn retire_org_tier_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, org_tier_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_org_tier_by_id(org_tier_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("org_tier", &r.org_tier_by_id);

    let rendered = data.tmpl.render("org_tier/org_tier_retire.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/org_tier/{org_tier_id}/retire")]
pub async fn retire_org_tier_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<RetireForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, org_tier_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/org_tier/{}", &lang, &org_tier_id));
    }

    let tier_data = update_org_tier::OrgTierData {
        id: org_tier_id.clone(),
        name_en: None,
        name_fr: None,
        tier_level: None,
        primary_domain: None,
        parent_tier: None,
        retired_at: Some(chrono::Utc::now().naive_utc()),
    };

    match update_org_tier(tier_data, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => {
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Organization tier retired.", "Niveau organisationnel retiré."),
            );
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
        },
    };

    redirect_to(format!("/{}/org_tier/{}", &lang, &org_tier_id))
}

#[derive(Deserialize, Debug)]
pub struct OwnerForm {
    pub csrf_token: String,
    pub person_name: String,
}

#[get("/{lang}/org_tier/{org_tier_id}/owner")]
pub async fn assign_org_owner_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, org_tier_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_org_tier_by_id(org_tier_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("org_tier", &r.org_tier_by_id);

    let rendered = data.tmpl.render("org_tier/assign_owner.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/org_tier/{org_tier_id}/owner")]
pub async fn assign_org_owner_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<OwnerForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, org_tier_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/org_tier/{}/owner", &lang, &org_tier_id));
    }

    match resolve_person_by_name(&form.person_name, &auth.bearer, &lang, &data).await {
        Ok(Some(person_id)) => {
            // Reassign if the tier already has an ownership record;
            // otherwise create one (tiers from createOrgTier have none).
            let existing = get_org_ownership_by_tier_id(org_tier_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await.ok();
            let result = if let Some(existing) = existing {
                update_org_ownership(update_org_ownership::OrgOwnershipData {
                    id: existing.org_ownership_by_tier_id.id,
                    owner_id: Some(person_id),
                    org_tier_id: None,
                    retired_at: None,
                }, auth.bearer, &data.api_url, Arc::clone(&data.client)).await.map(|_| ())
            } else {
                create_org_ownership(create_org_ownership::NewOrgOwnership {
                    owner_id: person_id,
                    org_tier_id: org_tier_id.clone(),
                }, auth.bearer, &data.api_url, Arc::clone(&data.client)).await.map(|_| ())
            };
            match result {
                Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Owner assigned.", "Responsable assigné.")),
                Err(e) => security::add_flash(&session, "danger", &e.to_string()),
            };
        },
        Ok(None) => security::add_flash(&session, "danger", by_lang(&lang, "Enter the owner's name.", "Entrez le nom du responsable.")),
        Err(message) => security::add_flash(&session, "danger", &message),
    };

    redirect_to(format!("/{}/org_tier/{}", &lang, &org_tier_id))
}

#[post("/{lang}/org_tier/{org_tier_id}/restore")]
pub async fn restore_org_tier_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<RetireForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, org_tier_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/org_tier/{}", &lang, &org_tier_id));
    }

    match restore_org_tier(org_tier_id.clone(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Organization tier restored.", "Niveau organisationnel restauré.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/org_tier/{}", &lang, &org_tier_id))
}
