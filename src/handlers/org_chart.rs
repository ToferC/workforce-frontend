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
use crate::graphql::{get_organization_by_id, get_org_tiers_by_org_id, get_org_tier_by_id, get_org_tier_node, get_team_by_id};
use crate::security::{self, MinimumRole};
use super::utility::{render_page, session_bearer};

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

    render_page(&data, "org_chart/builder.html", &ctx)
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

    render_page(&data, "org_chart/panel.html", &ctx)
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

    render_page(&data, "org_chart/node.html", &ctx)
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

    let bearer = session_bearer(&session);

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

// ── Visual org chart explorer (Phase 1) ─────────────────────────────────────
// A read-only counterpart to the builder. The whole tier+team skeleton is built
// from a single OrgTiersByOrgId query and handed to an ECharts tree on the
// client; team stats and the people behind roles load lazily in later phases.

/// A flattened, language-resolved tier used to build the chart JSON without
/// depending on the long generated GraphQL types.
struct TierLite {
    id: String,
    name: String,
    tier_level: i64,
    retired: bool,
    parent_id: Option<String>,
    primary_label: String,
    primary_group: String,
    teams: Vec<TeamLite>,
}

/// A team stub for the chart: identity plus the cheap server-computed capacity
/// aggregates used for the heatmap. People/roles are loaded lazily elsewhere.
struct TeamLite {
    id: String,
    name: String,
    headcount: i64,
    effort: i64,
}

/// Tiers at or above this level are leadership levels (L0 DM/CDS … L3
/// Director/Colonel): their "team" is a small leadership team that merges into
/// the tier box. Tier 4 is where the actual working teams live, and those stay
/// as their own boxes that drill down to roles and people.
const WORKING_TIER_LEVEL: i64 = 4;

/// Recursively build one tier's org-chart node.
///
/// Child tiers (sorted by level then name) are always nested below as their own
/// boxes. Teams are handled by level: for a leadership tier they are *merged*
/// into this box (`mergedTeams`, with headcount/effort aggregated onto the
/// tier), so the tier and its leadership team read as one unit; for a working
/// tier they become their own team leaf boxes (sorted by name).
fn build_tier_node(
    idx: usize,
    lite: &[TierLite],
    by_parent: &BTreeMap<String, Vec<usize>>,
) -> serde_json::Value {
    let t = &lite[idx];
    let leadership = t.tier_level < WORKING_TIER_LEVEL;

    let mut children: Vec<serde_json::Value> = Vec::new();
    if let Some(kids) = by_parent.get(&t.id) {
        let mut kids = kids.clone();
        kids.sort_by(|&a, &b| {
            lite[a].tier_level.cmp(&lite[b].tier_level).then(lite[a].name.cmp(&lite[b].name))
        });
        for k in kids {
            children.push(build_tier_node(k, lite, by_parent));
        }
    }

    let mut teams: Vec<&TeamLite> = t.teams.iter().collect();
    teams.sort_by(|a, b| a.name.cmp(&b.name));

    // Leadership tiers fold their team(s) into the box; working tiers keep them
    // as separate working-team boxes.
    let mut merged_teams: Vec<serde_json::Value> = Vec::new();
    let mut head_total: i64 = 0;
    let mut effort_total: i64 = 0;
    for team in teams {
        if leadership {
            head_total += team.headcount;
            effort_total += team.effort;
            merged_teams.push(json!({
                "id": team.id,
                "name": team.name,
                "headcount": team.headcount,
                "effort": team.effort,
            }));
        } else {
            children.push(json!({
                "id": team.id,
                "name": team.name,
                "kind": "team",
                "headcount": team.headcount,
                "effort": team.effort,
            }));
        }
    }

    json!({
        "id": t.id,
        "name": t.name,
        "kind": "tier",
        "tierLevel": t.tier_level,
        "leadership": leadership,
        "retired": t.retired,
        "primaryLabel": t.primary_label,
        "primaryGroup": t.primary_group,
        // Aggregated across the merged leadership team(s); 0 for working tiers.
        "headcount": head_total,
        "effort": effort_total,
        "mergedTeams": merged_teams,
        "children": children,
    })
}

/// Read-only visual explorer for one organization's structure.
#[get("/{lang}/organization/{organization_id}/org_chart/explore")]
pub async fn org_chart_explore(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, organization_id) = path_params.into_inner();
    let session = req.get_session();

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

    // Flatten with names resolved to the active language.
    let lite: Vec<TierLite> = tiers.iter().map(|t| {
        let primary = serde_json::to_value(&t.primary_domain)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        TierLite {
        id: t.id.to_string(),
        name: if lang == "fr" { t.name_fr.clone() } else { t.name_en.clone() },
        tier_level: t.tier_level,
        retired: t.retired_at.is_some(),
        parent_id: t.parent_organization_tier.as_ref().map(|p| p.id.to_string()),
        primary_label: domain_short_label(&primary).to_string(),
        primary_group: domain_group(&primary).to_string(),
        teams: t.teams.iter().map(|tm| TeamLite {
            id: tm.id.to_string(),
            name: tm.name_english.clone(),
            headcount: tm.headcount,
            effort: tm.total_effort,
        }).collect(),
        }
    }).collect();

    // Index children by parent id; roots have no parent.
    let mut by_parent: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    let mut roots: Vec<usize> = Vec::new();
    for (i, t) in lite.iter().enumerate() {
        match &t.parent_id {
            Some(pid) => by_parent.entry(pid.clone()).or_default().push(i),
            None => roots.push(i),
        }
    }
    roots.sort_by(|&a, &b| {
        lite[a].tier_level.cmp(&lite[b].tier_level).then(lite[a].name.cmp(&lite[b].name))
    });

    let root_nodes: Vec<serde_json::Value> = roots.iter()
        .map(|&i| build_tier_node(i, &lite, &by_parent))
        .collect();

    // The organization name for the synthetic root is read by the client from a
    // data attribute on the container, so we only ship the tier children here.
    let chart_data = json!({ "children": root_nodes });

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("organization", &organization);
    ctx.insert("organization_id", &organization_id);
    ctx.insert("orgchart_data", &chart_data);
    ctx.insert("tier_count", &tiers.len());

    render_page(&data, "org_chart/explore.html", &ctx)
}

/// JSON: the roles and people of one team, for the explorer's lazy drill-down.
/// Occupied roles carry their person; vacant roles are flagged. This is the
/// expensive leaf data, so it's only fetched when a team box is expanded.
#[get("/{lang}/team/{team_id}/members.json")]
pub async fn team_members_json(
    data: web::Data<AppData>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, team_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::User) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let team = match get_team_by_id(team_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.team_by_id,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({ "error": e.to_string() }));
        },
    };

    // Build leaf nodes keyed by role id, remembering each role's reports_to so
    // we can reconstruct the intra-team reporting hierarchy below. Order is
    // preserved (occupied first, then vacant) for stable roots.
    let mut nodes: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
    let mut reports: std::collections::HashMap<String, Option<String>> = std::collections::HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for role in &team.occupied_roles {
        let person = role.person.as_ref().map(|p| {
            // Person's top 3 capabilities, ranked by (validated, else
            // self-identified) level, rendered as domain-coloured chips.
            let mut scored: Vec<(i64, serde_json::Value)> = p.capabilities.iter().map(|c| {
                let domain = serde_json::to_value(&c.domain)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default();
                let level = c.validated_level.as_ref().or(Some(&c.self_identified_level));
                let weight = level
                    .and_then(|l| serde_json::to_value(l).ok())
                    .and_then(|v| v.as_str().map(|s| level_weight(s)))
                    .unwrap_or(0);
                let name = if lang == "fr" { c.name_fr.clone() } else { c.name_en.clone() };
                (weight, json!({ "label": name, "group": domain_group(&domain) }))
            }).collect();
            scored.sort_by(|a, b| b.0.cmp(&a.0));
            let caps: Vec<serde_json::Value> = scored.into_iter().take(3).map(|(_, v)| v).collect();

            json!({
                "id": p.id,
                "name": format!("{} {}", p.given_name, p.family_name),
                "capabilities": caps,
            })
        });
        let id = role.id.to_string();
        nodes.insert(id.clone(), json!({
            "kind": "role",
            "id": role.id,
            "title": role.title_english,
            "effort": role.effort,
            "vacant": false,
            "person": person,
        }));
        reports.insert(id.clone(), role.reports_to_id.clone());
        order.push(id);
    }
    for role in &team.vacant_roles {
        let id = role.id.to_string();
        nodes.insert(id.clone(), json!({
            "kind": "role",
            "id": role.id,
            "title": role.title_english,
            "vacant": true,
            "person": serde_json::Value::Null,
        }));
        reports.insert(id.clone(), role.reports_to_id.clone());
        order.push(id);
    }

    // A role is a child of its manager only when that manager is another role on
    // this same team; otherwise (reports_to is null, or points at a manager on
    // another team — e.g. the team owner) it is a root of this team's subtree.
    let member_ids: std::collections::HashSet<&String> = order.iter().collect();
    let mut children: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut roots: Vec<String> = Vec::new();
    for id in &order {
        match reports.get(id).cloned().flatten() {
            Some(pid) if pid != *id && member_ids.contains(&pid) => {
                children.entry(pid).or_default().push(id.clone());
            }
            _ => roots.push(id.clone()),
        }
    }

    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut out: Vec<serde_json::Value> = Vec::new();
    for r in &roots {
        if visited.insert(r.clone()) {
            out.push(assemble_member(r, &nodes, &children, &mut visited));
        }
    }

    HttpResponse::Ok().json(out)
}

/// Attach a role's direct reports (recursively) as its `children`, so the
/// explorer can render the reporting hierarchy inside a team. `visited` guards
/// against any cycle in the data so this can't recurse forever.
fn assemble_member(
    id: &str,
    nodes: &std::collections::HashMap<String, serde_json::Value>,
    children: &std::collections::HashMap<String, Vec<String>>,
    visited: &mut std::collections::HashSet<String>,
) -> serde_json::Value {
    let mut node = nodes.get(id).cloned().unwrap_or(serde_json::Value::Null);
    let mut kid_vals: Vec<serde_json::Value> = Vec::new();
    if let Some(kids) = children.get(id) {
        for k in kids {
            if visited.insert(k.clone()) {
                kid_vals.push(assemble_member(k, nodes, children, visited));
            }
        }
    }
    if let Some(obj) = node.as_object_mut() {
        obj.insert("children".to_string(), serde_json::Value::Array(kid_vals));
    }
    node
}
