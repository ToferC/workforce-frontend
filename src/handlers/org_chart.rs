// The org chart builder: a two-pane view where the right side is an
// expandable tree of org tiers (lazy-loaded per node with HTMX) down to
// teams, roles, and the people in them, and the left side is an info
// panel for the selected tier with edit / add-child actions.

use actix_session::{Session, SessionExt};
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use actix_identity::{Identity};
use serde_json::json;

use std::collections::BTreeMap;
use std::sync::Arc;
use crate::{AppData, generate_basic_context, domain_group, domain_short_label, level_weight};
use crate::graphql::{get_organization_by_id, get_org_tiers_by_org_id, get_org_tier_by_id, get_org_tier_node};
use crate::security::{self, MinimumRole};

/// Full-page builder view for one organization. The tree starts with the
/// organization's root tiers (no parent); each node lazy-loads its
/// children, teams, and roles when expanded.
#[get("/{lang}/organization/{organization_id}/org_chart")]
pub async fn org_chart_builder(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, organization_id) = path_params.into_inner();
    let session = req.get_session();

    // Any signed-in user can view the chart; mutating actions stay
    // operator-gated in their own handlers
    let auth = match security::require_role(&session, &lang, MinimumRole::User) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let organization = match get_organization_by_id(organization_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.organization_by_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return HttpResponse::Found()
                .append_header(("Location", format!("/{}", &lang)))
                .finish();
        },
    };

    let tiers = match get_org_tiers_by_org_id(organization_id.clone(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.org_tiers_by_org_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return HttpResponse::Found()
                .append_header(("Location", format!("/{}/organization/{}", &lang, &organization_id)))
                .finish();
        },
    };

    // Root tiers: no parent, sorted by tier level then name
    let mut roots: Vec<serde_json::Value> = tiers
        .iter()
        .filter(|tier| tier.parent_organization_tier.is_none())
        .map(|tier| json!({
            "id": tier.id,
            "nameEn": tier.name_en,
            "nameFr": tier.name_fr,
            "tierLevel": tier.tier_level,
            "retiredAt": tier.retired_at,
        }))
        .collect();
    roots.sort_by(|a, b| {
        let level = a["tierLevel"].as_i64().cmp(&b["tierLevel"].as_i64());
        level.then(a["nameEn"].as_str().cmp(&b["nameEn"].as_str()))
    });

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("organization", &organization);
    ctx.insert("organization_id", &organization_id);
    ctx.insert("root_tiers", &roots);
    ctx.insert("tier_count", &tiers.len());

    let rendered = data.tmpl.render("org_chart/builder.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// HTMX partial: the expanded body of one tier node — child tiers
/// (collapsed, lazy), teams with occupied/vacant roles, and an
/// add-child-tier action for operators.
#[get("/{lang}/org_tier/{org_tier_id}/node")]
pub async fn org_tier_node_partial(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, org_tier_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::User) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    render_node(&data, &session, id, &lang, &org_tier_id, &auth.bearer, &req).await
}

/// HTMX partial: tier details for the left-hand info panel.
#[get("/{lang}/org_tier/{org_tier_id}/panel")]
pub async fn org_tier_panel_partial(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, org_tier_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::User) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_org_tier_by_id(org_tier_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::Ok().body(format!("<div class=\"alert alert-danger\">{}</div>", e));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("org_tier", &r.org_tier_by_id);

    let rendered = data.tmpl.render("org_chart/panel.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

async fn render_node(
    data: &web::Data<AppData>,
    session: &Session,
    id: Option<Identity>,
    lang: &str,
    org_tier_id: &str,
    bearer: &str,
    req: &HttpRequest,
) -> HttpResponse {
    let node = match get_org_tier_node(org_tier_id.to_string(), bearer.to_string(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.org_tier_by_id,
        Err(e) => {
            return HttpResponse::Ok().body(format!("<div class=\"alert alert-danger\">{}</div>", e));
        },
    };

    // Build per-team capability/capacity stats for the org chart overlay.
    let mut team_stats_map = serde_json::Map::new();
    {
        let n = &node;
        for team in &n.teams {
            let headcount = team.occupied_roles.len() as i64;
            let vacant = team.vacant_roles.len() as i64;

            let mut total_effort: i64 = 0;
            let mut domain_depth: BTreeMap<String, i64> = BTreeMap::new();

            for role in &team.occupied_roles {
                if let Some(person) = &role.person {
                    total_effort += person.active_effort;
                    for cap in &person.capabilities {
                        let domain = serde_json::to_value(&cap.domain)
                            .ok()
                            .and_then(|v| v.as_str().map(String::from))
                            .unwrap_or_else(|| "UNKNOWN".to_string());
                        let level_val = cap.validated_level.as_ref().or(Some(&cap.self_identified_level));
                        if let Some(lvl) = level_val {
                            let level_str = serde_json::to_value(lvl)
                                .ok()
                                .and_then(|v| v.as_str().map(String::from))
                                .unwrap_or_else(|| "UNKNOWN".to_string());
                            *domain_depth.entry(domain).or_insert(0) += level_weight(&level_str);
                        }
                    }
                }
            }

            let mut sorted: Vec<(String, i64)> = domain_depth.into_iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(&a.1));

            let top_domains: Vec<serde_json::Value> = sorted.into_iter().take(3)
                .map(|(d, _)| json!({
                    "label": domain_short_label(&d),
                    "group": domain_group(&d),
                }))
                .collect();

            let capacity_class = if total_effort > 50 { "danger" }
                else if total_effort > 20 { "warning" }
                else { "success" };

            team_stats_map.insert(team.id.to_string(), json!({
                "headcount": headcount,
                "vacant": vacant,
                "effort": total_effort,
                "top_domains": top_domains,
                "capacity_class": capacity_class,
            }));
        }
    }

    let mut ctx = generate_basic_context(id, lang, req.uri().path(), session);
    ctx.insert("node", &node);
    ctx.insert("team_stats", &serde_json::Value::Object(team_stats_map));

    let rendered = data.tmpl.render("org_chart/node.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// Response for a successful HTMX create-tier post from the builder.
/// Re-renders the parent's node body and retargets the swap at it, so
/// the new child appears in the tree immediately. When the new tier is
/// a root, tells the client to reload the whole builder page instead.
pub async fn render_node_response(
    data: &web::Data<AppData>,
    session: &Session,
    id: Option<Identity>,
    lang: &str,
    parent_tier_id: &str,
    organization_id: &str,
    req: &HttpRequest,
) -> HttpResponse {
    if parent_tier_id.is_empty() {
        return HttpResponse::Ok()
            .append_header(("HX-Redirect", format!("/{}/organization/{}/org_chart", lang, organization_id)))
            .finish();
    }

    let bearer = match session.get::<String>("bearer") {
        Ok(Some(b)) => b,
        _ => String::new(),
    };

    let mut response = render_node(data, session, id, lang, parent_tier_id, &bearer, req).await;

    // Redirect the swap from the submitted form to the parent node's body
    let headers = response.headers_mut();
    headers.insert(
        actix_web::http::header::HeaderName::from_static("hx-retarget"),
        actix_web::http::header::HeaderValue::from_str(&format!("#node-body-{}", parent_tier_id)).unwrap(),
    );
    headers.insert(
        actix_web::http::header::HeaderName::from_static("hx-reswap"),
        actix_web::http::header::HeaderValue::from_static("innerHTML"),
    );

    response
}
