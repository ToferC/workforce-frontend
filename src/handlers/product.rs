use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::Identity;
use serde::Deserialize;
use serde_json::json;

use std::sync::Arc;
use crate::{AppData, generate_basic_context, by_lang};
use crate::graphql::{
    get_product_by_id, all_products, create_product, update_product,
    all_roles, all_organizations,
};
use crate::security::{self, MinimumRole};
use super::org_tier::{skill_domain_options, humanize};
use super::task::work_status_options;

fn redirect_to(location: String) -> HttpResponse {
    HttpResponse::Found().append_header(("Location", location)).finish()
}

fn csrf_failure_flash(session: &actix_session::Session, lang: &str) {
    security::add_flash(session, "danger", by_lang(lang, "Invalid form token. Please try again.", "Jeton de formulaire invalide. Veuillez réessayer."));
}

/// Build a JSON array of {value, label} from all active roles for use in a
/// product-owner select. Label format: "Given Family — Title (Team)" for
/// filled roles, "Vacant — Title (Team)" for unfilled.
async fn role_options(bearer: &str, data: &AppData) -> serde_json::Value {
    match all_roles(bearer.to_string(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => json!(r.all_roles
            .iter()
            .map(|role| {
                let person_prefix = role.person.as_ref()
                    .map(|p| format!("{} {} \u{2014} ", p.given_name, p.family_name))
                    .unwrap_or_else(|| "Vacant \u{2014} ".to_string());
                let team_suffix = format!(" ({})", role.team.name_english);
                json!({"value": role.id, "label": format!("{}{}{}", person_prefix, role.title_english, team_suffix)})
            })
            .collect::<Vec<_>>()),
        Err(_) => json!([]),
    }
}

pub async fn organization_options_json(bearer: &str, data: &AppData) -> serde_json::Value {
    match all_organizations(bearer.to_string(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => json!(r.all_organizations
            .iter()
            .map(|org| json!({"value": org.id, "label": org.name_en}))
            .collect::<Vec<serde_json::Value>>()),
        Err(_) => json!([]),
    }
}

#[derive(Deserialize, Debug)]
pub struct ProductForm {
    pub csrf_token: String,
    // Create only — org is immutable after creation (API has no organizationId in ProductData)
    #[serde(default)]
    pub organization_id: String,
    pub product_owner_role_id: String,
    pub name_en: String,
    pub name_fr: String,
    pub description_en: String,
    pub description_fr: String,
    pub primary_domain: String,
    #[serde(default)]
    pub url: String,
    pub product_status: String,
}

fn product_from_form(form: &ProductForm, id: Option<&str>) -> serde_json::Value {
    json!({
        "id": id,
        "nameEn": form.name_en,
        "nameFr": form.name_fr,
        "descriptionEn": form.description_en,
        "descriptionFr": form.description_fr,
        "primaryDomain": form.primary_domain,
        "url": form.url,
        "productStatus": form.product_status,
        "organization": {"id": form.organization_id},
        "productOwner": {"id": form.product_owner_role_id},
    })
}

#[get("/{lang}/products")]
pub async fn product_index(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let products = all_products(bearer, &data.api_url, Arc::clone(&data.client)).await
        .map(|r| r.all_products)
        .unwrap_or_default();
    ctx.insert("products", &products);

    let rendered = data.tmpl.render("product/product_index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/product/{product_id}")]
pub async fn product_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, product_id) = path_params.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_product_by_id(product_id, bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get product");

    ctx.insert("product", &r.product_by_id);

    let rendered = data.tmpl.render("product/product.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/product/new")]
pub async fn create_product_form(
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
    ctx.insert("product", &json!({
        "nameEn": "", "nameFr": "", "descriptionEn": "", "descriptionFr": "",
        "primaryDomain": "", "url": "", "productStatus": "PLANNING",
        "organization": {"id": ""}, "productOwner": {"id": ""},
    }));
    ctx.insert("organization_options", &organization_options_json(&auth.bearer, &data).await);
    ctx.insert("role_options", &role_options(&auth.bearer, &data).await);
    ctx.insert("skill_domains", &skill_domain_options());
    ctx.insert("work_statuses", &work_status_options());

    let rendered = data.tmpl.render("product/product_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/product/new")]
pub async fn create_product_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    form: web::Form<ProductForm>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/product/new", &lang));
    }

    let render_error = |message: String, id: Option<Identity>, org_opts: serde_json::Value, role_opts: serde_json::Value| {
        security::add_flash(&session, "danger", &message);
        let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
        ctx.insert("edit", &false);
        ctx.insert("product", &product_from_form(&form, None));
        ctx.insert("organization_options", &org_opts);
        ctx.insert("role_options", &role_opts);
        ctx.insert("skill_domains", &skill_domain_options());
        ctx.insert("work_statuses", &work_status_options());
        let rendered = data.tmpl.render("product/product_form.html", &ctx).unwrap();
        HttpResponse::Ok().body(rendered)
    };

    let new_product = create_product::NewProduct {
        organization_id: form.organization_id.clone(),
        product_owner_role_id: form.product_owner_role_id.clone(),
        name_en: form.name_en.trim().to_string(),
        name_fr: form.name_fr.trim().to_string(),
        description_en: form.description_en.trim().to_string(),
        description_fr: form.description_fr.trim().to_string(),
        primary_domain: serde_json::from_value(json!(form.primary_domain))
            .expect("SkillDomain deserialization is infallible"),
        url: if form.url.trim().is_empty() { None } else { Some(form.url.trim().to_string()) },
        product_status: serde_json::from_value(json!(form.product_status))
            .expect("WorkStatus deserialization is infallible"),
    };

    match create_product(new_product, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(&session, "success", by_lang(&lang, "Product created.", "Produit créé."));
            redirect_to(format!("/{}/product/{}", &lang, response.create_product.id))
        },
        Err(e) => {
            let org_opts = organization_options_json(&auth.bearer, &data).await;
            let role_opts = role_options(&auth.bearer, &data).await;
            render_error(e.to_string(), id, org_opts, role_opts)
        },
    }
}

#[get("/{lang}/product/{product_id}/edit")]
pub async fn edit_product_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, product_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_product_by_id(product_id, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/products", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &true);
    ctx.insert("product", &r.product_by_id);
    ctx.insert("role_options", &role_options(&auth.bearer, &data).await);
    ctx.insert("skill_domains", &skill_domain_options());
    ctx.insert("work_statuses", &work_status_options());

    let rendered = data.tmpl.render("product/product_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/product/{product_id}/edit")]
pub async fn edit_product_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<ProductForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, product_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/product/{}/edit", &lang, &product_id));
    }

    let product_data = update_product::ProductData {
        id: product_id.clone(),
        product_owner_role_id: Some(form.product_owner_role_id.clone()),
        name_en: Some(form.name_en.trim().to_string()),
        name_fr: Some(form.name_fr.trim().to_string()),
        description_en: Some(form.description_en.trim().to_string()),
        description_fr: Some(form.description_fr.trim().to_string()),
        primary_domain: Some(serde_json::from_value(json!(form.primary_domain))
            .expect("SkillDomain deserialization is infallible")),
        url: if form.url.trim().is_empty() { None } else { Some(form.url.trim().to_string()) },
        product_status: Some(serde_json::from_value(json!(form.product_status))
            .expect("WorkStatus deserialization is infallible")),
    };

    match update_product(product_data, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(&session, "success", by_lang(&lang, "Product updated.", "Produit mis à jour."));
            redirect_to(format!("/{}/product/{}", &lang, response.update_product.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &true);
            ctx.insert("product", &product_from_form(&form, Some(&product_id)));
            ctx.insert("role_options", &role_options(&auth.bearer, &data).await);
            ctx.insert("skill_domains", &skill_domain_options());
            ctx.insert("work_statuses", &work_status_options());
            let rendered = data.tmpl.render("product/product_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}

/// Returns all products as {value, label} pairs for use in task form selects.
pub async fn product_options(bearer: &str, data: &AppData) -> serde_json::Value {
    match all_products(bearer.to_string(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => json!(r.all_products
            .iter()
            .map(|p| {
                let domain_str = serde_json::to_value(&p.primary_domain)
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_default();
                let label = format!("{} ({})", p.name_en, humanize(&domain_str));
                json!({"value": p.id, "label": label})
            })
            .collect::<Vec<_>>()),
        Err(_) => json!([]),
    }
}
