use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use actix_identity::Identity;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::{AppData, generate_basic_context};
use crate::graphql::{all_work, vacant_roles, analytics_people, analytics_roles};
use crate::security::{self, MinimumRole};

#[get("/{lang}/analytics")]
pub async fn analytics_dashboard(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Analyst) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    // Fetch all four data sources; tolerate individual failures gracefully.
    let work_list = all_work(auth.bearer.clone(), &data.api_url, Arc::clone(&data.client))
        .await
        .map(|r| r.all_work)
        .unwrap_or_default();

    let vacant_role_list = vacant_roles(200, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client))
        .await
        .map(|r| r.vacant_roles)
        .unwrap_or_default();

    let people_list = analytics_people(auth.bearer.clone(), &data.api_url, Arc::clone(&data.client))
        .await
        .map(|r| r.all_people)
        .unwrap_or_default();

    let roles_list = analytics_roles(auth.bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .map(|r| r.all_roles)
        .unwrap_or_default();

    // ── 1. Work status summary ──────────────────────────────────────────────
    let total_work = work_list.len() as i64;
    let vacant_work_count = work_list.iter().filter(|w| w.role.is_none()).count() as i64;

    let mut status_map: BTreeMap<String, i64> = BTreeMap::new();
    for w in &work_list {
        let status = serde_json::to_value(&w.work_status)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "UNKNOWN".to_string());
        *status_map.entry(status).or_insert(0) += 1;
    }
    let work_status_counts: Vec<serde_json::Value> = status_map
        .into_iter()
        .map(|(status, count)| json!({"status": status, "count": count}))
        .collect();

    // ── 2. Capacity / utilization ───────────────────────────────────────────
    let active_people: Vec<_> = people_list.iter()
        .filter(|p| p.retired_at.is_none())
        .collect();

    let total_people = active_people.len() as i64;
    let overloaded_threshold = 8i64;

    let mut over_allocated: Vec<serde_json::Value> = active_people.iter()
        .filter(|p| p.active_effort >= overloaded_threshold)
        .map(|p| {
            let team = p.active_roles.first()
                .map(|r| r.team.name_english.clone())
                .unwrap_or_else(|| "No team".to_string());
            json!({
                "id": p.id,
                "name": format!("{} {}", p.given_name, p.family_name),
                "team": team,
                "effort": p.active_effort,
            })
        })
        .collect();
    over_allocated.sort_by(|a, b| {
        b["effort"].as_i64().unwrap_or(0).cmp(&a["effort"].as_i64().unwrap_or(0))
    });

    let available_count = active_people.iter()
        .filter(|p| p.active_effort == 0)
        .count() as i64;

    // Team effort summary
    let mut team_effort: BTreeMap<String, i64> = BTreeMap::new();
    for person in &active_people {
        let team = person.active_roles.first()
            .map(|r| r.team.name_english.clone())
            .unwrap_or_else(|| "Unattached".to_string());
        *team_effort.entry(team).or_insert(0) += person.active_effort;
    }
    let mut team_capacity: Vec<serde_json::Value> = team_effort
        .into_iter()
        .map(|(team, effort)| json!({"team": team, "effort": effort, "overloaded": effort > 50}))
        .collect();
    team_capacity.sort_by(|a, b| {
        b["effort"].as_i64().unwrap_or(0).cmp(&a["effort"].as_i64().unwrap_or(0))
    });

    // ── 3. Capability gap analysis ──────────────────────────────────────────
    // Required: count by (domain, level) from role requirements
    let mut required: BTreeMap<String, BTreeMap<String, i64>> = BTreeMap::new();
    for role in &roles_list {
        for req in &role.requirements {
            let domain = serde_json::to_value(&req.domain)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "UNKNOWN".to_string());
            let level = serde_json::to_value(&req.required_level)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "UNKNOWN".to_string());
            *required.entry(domain).or_default().entry(level).or_insert(0) += 1;
        }
    }

    // Available: count by (domain, validated_level) from person capabilities
    let mut available: BTreeMap<String, BTreeMap<String, i64>> = BTreeMap::new();
    for person in &active_people {
        for cap in &person.capabilities {
            let domain = serde_json::to_value(&cap.domain)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "UNKNOWN".to_string());
            // Use validated level; fall back to self-identified if not yet validated
            let level_val = cap.validated_level.as_ref().or(Some(&cap.self_identified_level));
            if let Some(lvl) = level_val {
                let level = serde_json::to_value(lvl)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_else(|| "UNKNOWN".to_string());
                *available.entry(domain).or_default().entry(level).or_insert(0) += 1;
            }
        }
    }

    // Build domain gap table — all domains that appear in either required or available
    let all_domains: std::collections::BTreeSet<String> = required.keys()
        .chain(available.keys())
        .cloned()
        .collect();

    let all_levels = ["DESIRED", "NOVICE", "EXPERIENCED", "EXPERT", "SPECIALIST"];

    let mut domain_gaps: Vec<serde_json::Value> = all_domains.into_iter().map(|domain| {
        let req_by_level = required.get(&domain).cloned().unwrap_or_default();
        let avail_by_level = available.get(&domain).cloned().unwrap_or_default();

        let total_req: i64 = req_by_level.values().sum();
        let total_avail: i64 = avail_by_level.values().sum();
        let net_gap = total_req - total_avail;

        let level_detail: Vec<serde_json::Value> = all_levels.iter()
            .filter(|&&lvl| req_by_level.contains_key(lvl) || avail_by_level.contains_key(lvl))
            .map(|&lvl| {
                let req_count = *req_by_level.get(lvl).unwrap_or(&0);
                let avail_count = *avail_by_level.get(lvl).unwrap_or(&0);
                json!({
                    "level": lvl,
                    "required": req_count,
                    "available": avail_count,
                    "gap": req_count - avail_count,
                })
            })
            .collect();

        json!({
            "domain": domain,
            "total_required": total_req,
            "total_available": total_avail,
            "net_gap": net_gap,
            "shortfall": net_gap > 0,
            "levels": level_detail,
        })
    }).collect();

    // Sort: shortfalls first, then by gap descending
    domain_gaps.sort_by(|a, b| {
        let a_gap = a["net_gap"].as_i64().unwrap_or(0);
        let b_gap = b["net_gap"].as_i64().unwrap_or(0);
        b_gap.cmp(&a_gap)
    });

    // ── 4. Work domain effort distribution ─────────────────────────────────
    let mut domain_work_effort: BTreeMap<String, i64> = BTreeMap::new();
    for w in &work_list {
        let domain = serde_json::to_value(&w.domain)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "UNKNOWN".to_string());
        *domain_work_effort.entry(domain).or_insert(0) += w.effort;
    }
    let mut work_by_domain: Vec<serde_json::Value> = domain_work_effort
        .into_iter()
        .map(|(domain, effort)| json!({"domain": domain, "effort": effort}))
        .collect();
    work_by_domain.sort_by(|a, b| {
        b["effort"].as_i64().unwrap_or(0).cmp(&a["effort"].as_i64().unwrap_or(0))
    });

    // ── Assemble summary KPIs ───────────────────────────────────────────────
    let summary = json!({
        "total_work": total_work,
        "vacant_work": vacant_work_count,
        "vacant_roles": vacant_role_list.len() as i64,
        "total_people": total_people,
        "over_allocated_count": over_allocated.len() as i64,
        "available_count": available_count,
    });

    ctx.insert("summary", &summary);
    ctx.insert("work_status_counts", &work_status_counts);
    ctx.insert("team_capacity", &team_capacity);
    ctx.insert("over_allocated", &over_allocated);
    ctx.insert("domain_gaps", &domain_gaps);
    ctx.insert("work_by_domain", &work_by_domain);

    let rendered = data.tmpl.render("analytics/analytics.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}
