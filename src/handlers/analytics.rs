use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use actix_identity::Identity;
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use crate::{AppData, generate_basic_context, status_color, chart_json, domain_short_label};
use crate::graphql::{all_work, vacant_roles, analytics_people, analytics_roles, delivery_treemap,
    team_capability_matrix, talent_movements, capability_growth, capability_supply_demand, all_teams,
    all_org_tiers, priority_mismatches};

use crate::security::{self, MinimumRole};
use super::utility::{render_page};

const SKILL_DOMAINS: &[(&str, &str)] = &[
    ("COMBAT",                                "Combat"),
    ("INTELLIGENCE",                          "Intelligence"),
    ("STRATEGY",                              "Strategy"),
    ("ENGINEERING",                           "Engineering"),
    ("MEDICAL",                               "Medical"),
    ("JOINT_OPERATIONS",                      "Joint Ops"),
    ("SOFTWARE_ENGINEERING",                  "Software Eng"),
    ("CLOUD_PLATFORM_DEV_OPS",               "Cloud/DevOps"),
    ("DATA_ANALYTICS_AND_AI",                "Data & AI"),
    ("CYBER_SECURITY",                        "Cyber Security"),
    ("PRODUCT_AGILE_AND_DELIVERY",           "Product/Agile"),
    ("USER_EXPERIENCE",                       "UX"),
    ("PROCUREMENT_AND_VENDOR_MANAGEMENT",    "Procurement"),
    ("PEOPLE_AND_ORGANISATIONAL_LEADERSHIP", "People & Org"),
    ("GOVERNANCE",                            "Governance"),
    ("CORPORATE_SERVICES",                    "Corporate Svcs"),
];


#[get("/{lang}/analytics")]
pub async fn analytics_dashboard(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    // Enforce access, but make no API calls here: the shell renders instantly
    // and each section lazy-loads its own data via HTMX (see the fragment
    // handlers below). This keeps any single request well under Heroku's 30s
    // limit even when the underlying GraphQL queries are slow.
    if let Err(response) = security::require_role(&session, &lang, MinimumRole::Analyst) {
        return response;
    }

    let ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    render_page(&data, "analytics/analytics.html", &ctx)
}

// ── Dashboard section fragments ─────────────────────────────────────────────
// Each fragment is fetched independently by HTMX (hx-trigger="load") so the
// dashboard's previously-monolithic four-query render is split into separate,
// smaller requests that load in parallel from the browser.

#[get("/{lang}/analytics/section/work")]
pub async fn analytics_section_work(
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

    let work_list = all_work(None, false, None, 0, auth.bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .map(|r| r.all_work)
        .unwrap_or_default();

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

    ctx.insert("total_work", &total_work);
    ctx.insert("vacant_work", &vacant_work_count);
    ctx.insert("work_status_counts", &work_status_counts);
    ctx.insert("work_by_domain", &work_by_domain);

    render_page(&data, "analytics/_section_work.html", &ctx)
}

#[get("/{lang}/analytics/section/capacity")]
pub async fn analytics_section_capacity(
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

    let people_list = analytics_people(auth.bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .map(|r| r.all_people)
        .unwrap_or_default();

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

    ctx.insert("total_people", &total_people);
    ctx.insert("over_allocated_count", &(over_allocated.len() as i64));
    ctx.insert("available_count", &available_count);
    ctx.insert("team_capacity", &team_capacity);
    ctx.insert("over_allocated", &over_allocated);

    render_page(&data, "analytics/_section_capacity.html", &ctx)
}

#[get("/{lang}/analytics/section/vacancies")]
pub async fn analytics_section_vacancies(
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

    let vacant_role_list = vacant_roles(200, auth.bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .map(|r| r.vacant_roles)
        .unwrap_or_default();

    let rows: Vec<serde_json::Value> = vacant_role_list.iter()
        .map(|r| json!({
            "id": r.id,
            "title": r.title_english,
            "team": r.team.name_english,
        }))
        .collect();

    ctx.insert("vacant_roles_count", &(rows.len() as i64));
    ctx.insert("vacant_roles", &rows);

    render_page(&data, "analytics/_section_vacancies.html", &ctx)
}

#[get("/{lang}/analytics/section/gaps")]
pub async fn analytics_section_gaps(
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

    // Gaps compare role requirements against people's capabilities, so this one
    // section needs both datasets — fetch them concurrently.
    let (people_res, roles_res) = futures::future::join(
        analytics_people(auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)),
        analytics_roles(auth.bearer, &data.api_url, Arc::clone(&data.client)),
    ).await;

    let people_list = people_res.map(|r| r.all_people).unwrap_or_default();
    let roles_list = roles_res.map(|r| r.all_roles).unwrap_or_default();

    let active_people: Vec<_> = people_list.iter()
        .filter(|p| p.retired_at.is_none())
        .collect();

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

    let mut available: BTreeMap<String, BTreeMap<String, i64>> = BTreeMap::new();
    for person in &active_people {
        for cap in &person.capabilities {
            let domain = serde_json::to_value(&cap.domain)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "UNKNOWN".to_string());
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

    domain_gaps.sort_by(|a, b| {
        let a_gap = a["net_gap"].as_i64().unwrap_or(0);
        let b_gap = b["net_gap"].as_i64().unwrap_or(0);
        b_gap.cmp(&a_gap)
    });

    ctx.insert("domain_gaps", &domain_gaps);

    render_page(&data, "analytics/_section_gaps.html", &ctx)
}

#[get("/{lang}/analytics/coverage")]
pub async fn analytics_coverage(
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

    let matrix = team_capability_matrix(None, auth.bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .map(|r| r.team_capability_matrix)
        .unwrap_or_default();

    // Build depth_map from API response
    let mut depth_map: BTreeMap<String, BTreeMap<String, f64>> = BTreeMap::new();
    let mut team_ids: BTreeMap<String, String> = BTreeMap::new();

    for row in &matrix {
        let team_name = row.team_name.clone();
        team_ids.insert(team_name.clone(), row.team_id.clone());
        for cell in &row.cells {
            *depth_map
                .entry(team_name.clone())
                .or_default()
                .entry(cell.domain.clone())
                .or_insert(0.0) += cell.depth;
        }
    }

    let active_domains: Vec<(&str, &str)> = SKILL_DOMAINS.iter()
        .filter(|(key, _)| depth_map.values().any(|m| m.get(*key).copied().unwrap_or(0.0) > 0.0))
        .copied()
        .collect();

    let domain_labels: Vec<&str> = active_domains.iter().map(|(_, label)| *label).collect();
    let team_names: Vec<&str> = depth_map.keys().map(String::as_str).collect();

    let max_depth: f64 = depth_map.values()
        .flat_map(|m| m.values())
        .copied()
        .fold(1.0_f64, f64::max);

    let mut heatmap_data: Vec<serde_json::Value> = Vec::new();
    for (t_idx, team_name) in team_names.iter().enumerate() {
        for (d_idx, (domain_key, _)) in active_domains.iter().enumerate() {
            let depth = depth_map.get(*team_name)
                .and_then(|m| m.get(*domain_key))
                .copied()
                .unwrap_or(0.0);
            heatmap_data.push(json!([d_idx, t_idx, (depth * 10.0).round() / 10.0]));
        }
    }

    let chart_option = json!({
        "animation": false,
        "tooltip": { "position": "top" },
        "grid": { "top": "5%", "left": "20%", "right": "5%", "bottom": "25%" },
        "xAxis": {
            "type": "category",
            "data": domain_labels,
            "axisLabel": { "rotate": 35, "fontSize": 11 }
        },
        "yAxis": { "type": "category", "data": team_names },
        "visualMap": {
            "min": 0,
            "max": max_depth,
            "calculable": true,
            "orient": "horizontal",
            "left": "center",
            "bottom": "2%",
            "inRange": { "color": ["#f0f0f0", "#0a6d2e"] }
        },
        "series": [{
            "type": "heatmap",
            "data": heatmap_data,
            "label": { "show": true, "fontSize": 10 }
        }]
    });

    let table_rows: Vec<serde_json::Value> = depth_map.iter().map(|(team, domains)| {
        let team_id = team_ids.get(team).cloned().unwrap_or_default();
        let cells: Vec<serde_json::Value> = active_domains.iter().map(|(key, _)| {
            let depth = domains.get(*key).copied().unwrap_or(0.0);
            let opacity = if max_depth > 0.0 { depth / max_depth } else { 0.0 };
            json!({ "depth": (depth * 10.0).round() / 10.0, "opacity": opacity })
        }).collect();
        json!({ "team": team, "team_id": team_id, "cells": cells })
    }).collect();

    let mut domain_totals: Vec<serde_json::Value> = active_domains.iter().map(|(key, label)| {
        let total: f64 = depth_map.values()
            .map(|m| m.get(*key).copied().unwrap_or(0.0))
            .sum();
        json!({ "domain": label, "key": key, "total": (total * 10.0).round() / 10.0 })
    }).collect();
    domain_totals.sort_by(|a, b| {
        b["total"].as_f64().unwrap_or(0.0).partial_cmp(&a["total"].as_f64().unwrap_or(0.0)).unwrap()
    });

    let chart_height = format!("{}px", std::cmp::max(400, team_names.len() as i64 * 52 + 260));

    ctx.insert("chart_option", &chart_json(&chart_option));
    ctx.insert("chart_height", &chart_height);
    ctx.insert("table_rows", &table_rows);
    ctx.insert("domain_labels", &domain_labels);
    ctx.insert("domain_totals", &domain_totals);
    ctx.insert("summary", &json!({
        "total_teams": team_names.len() as i64,
        "active_domains": active_domains.len() as i64,
        "max_depth": (max_depth * 10.0).round() / 10.0,
    }));

    render_page(&data, "analytics/coverage.html", &ctx)
}

#[get("/{lang}/analytics/delivery")]
pub async fn analytics_delivery(
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

    let products = delivery_treemap(auth.bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .map(|r| r.all_products)
        .unwrap_or_default();

    let mut tree: Vec<serde_json::Value> = Vec::new();
    let mut total_tasks: i64 = 0;
    let mut total_work: i64 = 0;
    let mut total_effort: i64 = 0;
    let mut status_effort: BTreeMap<String, i64> = BTreeMap::new();
    let mut product_rows: Vec<serde_json::Value> = Vec::new();

    for product in &products {
        let mut task_nodes: Vec<serde_json::Value> = Vec::new();
        let mut product_value: i64 = 0;
        let mut product_work_count: i64 = 0;

        for task in &product.tasks {
            total_tasks += 1;
            let mut work_nodes: Vec<serde_json::Value> = Vec::new();
            let mut task_value: i64 = 0;

            for work in &task.work {
                total_work += 1;
                product_work_count += 1;
                let effort = std::cmp::max(work.effort, 1);
                task_value += effort;
                total_effort += work.effort;

                let status = serde_json::to_value(&work.work_status)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_else(|| "PLANNING".to_string());
                *status_effort.entry(status.clone()).or_insert(0) += work.effort;

                work_nodes.push(json!({
                    "name": work.work_description,
                    "value": effort,
                    "itemStyle": { "color": status_color(&status) },
                    "url": format!("/{}/work/{}", lang, work.id),
                }));
            }

            if task_value == 0 {
                task_value = std::cmp::max(task.effort, 1);
            }
            product_value += task_value;

            task_nodes.push(json!({
                "name": task.title,
                "value": task_value,
                "children": work_nodes,
                "url": format!("/{}/task/{}", lang, task.id),
            }));
        }

        if product_value == 0 {
            product_value = std::cmp::max(product.effort, 1);
        }

        let domain = serde_json::to_value(&product.primary_domain)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();

        product_rows.push(json!({
            "id": product.id,
            "name": product.name_en,
            "domain": domain,
            "effort": product.effort,
            "task_count": product.tasks.len() as i64,
            "work_count": product_work_count,
        }));

        if !task_nodes.is_empty() {
            tree.push(json!({
                "name": product.name_en,
                "value": product_value,
                "children": task_nodes,
                "url": format!("/{}/product/{}", lang, product.id),
            }));
        }
    }

    product_rows.sort_by(|a, b| {
        b["effort"].as_i64().unwrap_or(0).cmp(&a["effort"].as_i64().unwrap_or(0))
    });

    let chart_option = json!({
        "tooltip": { "formatter": "{b}: {c}" },
        "series": [{
            "type": "treemap",
            "roam": false,
            "nodeClick": "zoomToNode",
            "breadcrumb": { "show": true, "top": "5%" },
            "upperLabel": { "show": true, "height": 24, "color": "#fff" },
            "label": { "show": true, "formatter": "{b}" },
            "levels": [
                { "itemStyle": { "borderColor": "#333", "borderWidth": 4, "gapWidth": 4 } },
                { "itemStyle": { "borderColor": "#555", "borderWidth": 2, "gapWidth": 2 },
                  "upperLabel": { "show": true } },
                { "itemStyle": { "gapWidth": 1, "borderColorSaturation": 0.4 } }
            ],
            "data": tree,
        }]
    });

    let status_order = ["PLANNING", "IN_PROGRESS", "COMPLETED", "BLOCKED", "CANCELLED"];
    let status_legend: Vec<serde_json::Value> = status_order.iter()
        .filter(|s| status_effort.contains_key(**s))
        .map(|s| json!({
            "status": s,
            "color": status_color(s),
            "effort": status_effort.get(*s).copied().unwrap_or(0),
        }))
        .collect();

    ctx.insert("chart_option", &chart_json(&chart_option));
    ctx.insert("product_rows", &product_rows);
    ctx.insert("status_legend", &status_legend);
    ctx.insert("summary", &json!({
        "total_products": products.len() as i64,
        "rendered_products": tree.len() as i64,
        "total_tasks": total_tasks,
        "total_work": total_work,
        "total_effort": total_effort,
    }));

    render_page(&data, "analytics/delivery.html", &ctx)
}

/// Priority-consistency review (Proposal 7c): lists tasks whose priority is out
/// of step with the tiers around them — ranked below their product, or holding
/// work ranked below the task — so planners can realign them.
#[get("/{lang}/analytics/consistency")]
pub async fn analytics_consistency(
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

    let mismatches = priority_mismatches(auth.bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .map(|r| r.priority_mismatches)
        .unwrap_or_default();

    let mut rows: Vec<serde_json::Value> = Vec::new();
    let mut below_product_count: i64 = 0;
    let mut below_work_total: i64 = 0;

    for m in &mismatches {
        if m.task_below_product {
            below_product_count += 1;
        }
        below_work_total += m.below_work_count as i64;

        let task_priority = serde_json::to_value(&m.task_priority)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        let product_priority = m.product_priority.as_ref()
            .and_then(|p| serde_json::to_value(p).ok())
            .and_then(|v| v.as_str().map(String::from));

        rows.push(json!({
            "task_id": m.task_id,
            "task_title": m.task_title,
            "task_priority": task_priority,
            "product_id": m.product_id,
            "product_name": m.product_name,
            "product_priority": product_priority,
            "task_below_product": m.task_below_product,
            "below_work_count": m.below_work_count,
        }));
    }

    ctx.insert("rows", &rows);
    ctx.insert("summary", &json!({
        "total": mismatches.len() as i64,
        "below_product_count": below_product_count,
        "below_work_total": below_work_total,
    }));

    render_page(&data, "analytics/consistency.html", &ctx)
}

#[get("/{lang}/analytics/mobility")]
pub async fn analytics_mobility_view(
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

    // Movement events only carry team UUIDs. Resolve those to a named org
    // tier — and specifically to the tier one level below the root(s),
    // walking up each team's org tier parent chain — rather than each team's
    // own (often near-1:1-with-team) immediate tier. That keeps the sankey at
    // a readable, "command level" granularity instead of leaking UUIDs or
    // drawing hundreds of near-unique team-to-team links.
    let (movements_res, teams_res, org_tiers_res) = futures::join!(
        talent_movements(None, None, None, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)),
        all_teams(None, false, Some(5000), 0, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)),
        all_org_tiers(auth.bearer, &data.api_url, Arc::clone(&data.client)),
    );

    let movements = movements_res.map(|r| r.talent_movements).unwrap_or_default();

    let team_immediate_tier: HashMap<String, String> = teams_res
        .map(|r| r.all_teams.into_iter()
            .map(|t| (t.id, t.organization_level.id))
            .collect())
        .unwrap_or_default();

    let mut tier_name: HashMap<String, String> = HashMap::new();
    let mut tier_parent: HashMap<String, Option<String>> = HashMap::new();
    if let Ok(r) = org_tiers_res {
        for t in r.all_org_tiers {
            tier_parent.insert(t.id.clone(), t.parent_tier);
            tier_name.insert(t.id, t.name_en);
        }
    }

    // Walk a tier's parent chain up to (but not past) the tier just below the
    // root(s) of the hierarchy.
    fn top_level_tier_name(
        tier_id: &str,
        tier_parent: &HashMap<String, Option<String>>,
        tier_name: &HashMap<String, String>,
    ) -> Option<String> {
        let mut current = tier_id.to_string();
        loop {
            match tier_parent.get(&current) {
                Some(Some(parent)) => {
                    let parent_is_root = !matches!(tier_parent.get(parent), Some(Some(_)));
                    if parent_is_root {
                        break;
                    }
                    current = parent.clone();
                },
                _ => break,
            }
        }
        tier_name.get(&current).cloned()
    }

    let resolve_tier = |team_id: &Option<String>| -> String {
        team_id.as_ref()
            .and_then(|id| team_immediate_tier.get(id))
            .and_then(|tier_id| top_level_tier_name(tier_id, &tier_parent, &tier_name))
            .unwrap_or_else(|| "External".to_string())
    };

    // Build sankey from API-provided movement events, aggregated by org tier.
    let mut links: BTreeMap<(String, String), i64> = BTreeMap::new();
    let mut tiers_seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut kind_counts: BTreeMap<String, i64> = BTreeMap::new();

    for m in &movements {
        *kind_counts.entry(m.kind.clone()).or_insert(0) += 1;

        let from = resolve_tier(&m.from_team_id);
        let to = resolve_tier(&m.to_team_id);
        if from != to {
            tiers_seen.insert(from.clone());
            tiers_seen.insert(to.clone());
            *links.entry((from, to)).or_insert(0) += 1;
        }
    }

    let mut node_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for (o, d) in links.keys() {
        node_names.insert(format!("{} (was)", o));
        node_names.insert(format!("{} (now)", d));
    }
    let nodes: Vec<serde_json::Value> = node_names.iter()
        .map(|n| json!({"name": n}))
        .collect();

    let link_data: Vec<serde_json::Value> = links.iter()
        .map(|((o, d), count)| json!({
            "source": format!("{} (was)", o),
            "target": format!("{} (now)", d),
            "value": count,
        }))
        .collect();

    let total_moves: i64 = movements.len() as i64;
    let has_moves = !links.is_empty();

    let chart_option = json!({
        "tooltip": {"trigger": "item", "triggerOn": "mousemove"},
        "series": [{
            "type": "sankey",
            "layout": "none",
            "emphasis": {"focus": "adjacency"},
            "nodeAlign": "justify",
            "data": nodes,
            "links": link_data,
            "label": {"fontSize": 11},
            "lineStyle": {"color": "gradient", "curveness": 0.5},
        }]
    });

    let mut table_rows: Vec<serde_json::Value> = links.iter()
        .map(|((o, d), count)| json!({"from": o, "to": d, "count": count}))
        .collect();
    table_rows.sort_by(|a, b| {
        b["count"].as_i64().unwrap_or(0).cmp(&a["count"].as_i64().unwrap_or(0))
    });

    let promotions = kind_counts.get("PROMOTION").copied().unwrap_or(0);
    let laterals = kind_counts.get("LATERAL").copied().unwrap_or(0);
    let inflows = kind_counts.get("INFLOW").copied().unwrap_or(0);
    let outflows = kind_counts.get("OUTFLOW").copied().unwrap_or(0);

    ctx.insert("chart_option", &chart_json(&chart_option));
    ctx.insert("table_rows", &table_rows);
    ctx.insert("has_moves", &has_moves);
    ctx.insert("summary", &json!({
        "total_moves": total_moves,
        "promotions": promotions,
        "laterals": laterals,
        "inflows": inflows,
        "outflows": outflows,
        "org_tiers_involved": tiers_seen.len() as i64,
    }));

    render_page(&data, "analytics/mobility.html", &ctx)
}

#[get("/{lang}/analytics/growth")]
pub async fn analytics_growth(
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

    use crate::graphql::capability_growth::TimeBucket;

    let series = capability_growth(TimeBucket::QUARTER, None, None, None, auth.bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .map(|r| r.capability_growth)
        .unwrap_or_default();

    let domain_colors = [
        "#e6194b", "#3cb44b", "#4363d8", "#f58231", "#911eb4",
        "#42d4f4", "#f032e6", "#bfef45", "#fabed4", "#469990",
        "#dcbeff", "#9A6324", "#800000", "#aaffc3", "#808000",
        "#000075",
    ];

    let mut echarts_series: Vec<serde_json::Value> = Vec::new();
    let mut legend_data: Vec<String> = Vec::new();

    for (i, s) in series.iter().enumerate() {
        let label = domain_short_label(&s.key);
        legend_data.push(label.to_string());

        let data_points: Vec<serde_json::Value> = s.points.iter()
            .map(|p| json!([&p.period_start, p.value]))
            .collect();

        echarts_series.push(json!({
            "name": label,
            "type": "line",
            "smooth": true,
            "symbol": "circle",
            "symbolSize": 6,
            "data": data_points,
            "itemStyle": { "color": domain_colors.get(i % domain_colors.len()).unwrap_or(&"#333") },
        }));
    }

    let chart_option = json!({
        "tooltip": { "trigger": "axis" },
        "legend": { "data": legend_data, "bottom": "0%", "type": "scroll" },
        "grid": { "top": "8%", "left": "8%", "right": "5%", "bottom": "18%" },
        "xAxis": { "type": "time" },
        "yAxis": { "type": "value", "name": "Capability Depth" },
        "series": echarts_series,
    });

    let total_domains = series.len() as i64;
    let latest_total: f64 = series.iter()
        .filter_map(|s| s.points.last().map(|p| p.value))
        .sum();

    ctx.insert("chart_option", &chart_json(&chart_option));
    ctx.insert("summary", &json!({
        "total_domains": total_domains,
        "latest_total": (latest_total * 10.0).round() / 10.0,
    }));

    render_page(&data, "analytics/growth.html", &ctx)
}

#[get("/{lang}/analytics/supply-demand")]
pub async fn analytics_supply_demand(
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

    use crate::graphql::capability_supply_demand::TimeBucket;

    let series = capability_supply_demand(TimeBucket::QUARTER, None, None, None, auth.bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .map(|r| r.capability_supply_demand)
        .unwrap_or_default();

    // One sub-chart per domain, rendered as a grid of small multiples
    let mut domain_charts: Vec<serde_json::Value> = Vec::new();

    for s in &series {
        let label = domain_short_label(&s.domain);

        let supply_data: Vec<serde_json::Value> = s.points.iter()
            .map(|p| json!([&p.period_start, p.supply]))
            .collect();
        let demand_data: Vec<serde_json::Value> = s.points.iter()
            .map(|p| json!([&p.period_start, p.demand]))
            .collect();

        let latest_supply = s.points.last().map(|p| p.supply).unwrap_or(0.0);
        let latest_demand = s.points.last().map(|p| p.demand).unwrap_or(0.0);
        let gap = latest_supply - latest_demand;

        let chart_opt = json!({
            "tooltip": { "trigger": "axis" },
            "legend": { "data": ["Supply", "Demand"], "bottom": "0%" },
            "grid": { "top": "12%", "left": "12%", "right": "5%", "bottom": "18%" },
            "xAxis": { "type": "time", "axisLabel": { "fontSize": 10 } },
            "yAxis": { "type": "value" },
            "series": [
                {
                    "name": "Supply",
                    "type": "line",
                    "smooth": true,
                    "data": supply_data,
                    "areaStyle": { "opacity": 0.15 },
                    "itemStyle": { "color": "#3cb44b" },
                },
                {
                    "name": "Demand",
                    "type": "line",
                    "smooth": true,
                    "data": demand_data,
                    "lineStyle": { "type": "dashed" },
                    "itemStyle": { "color": "#e6194b" },
                }
            ]
        });

        domain_charts.push(json!({
            "domain": label,
            "domain_key": s.domain,
            "chart_option": chart_json(&chart_opt),
            "latest_supply": (latest_supply * 10.0).round() / 10.0,
            "latest_demand": (latest_demand * 10.0).round() / 10.0,
            "gap": (gap * 10.0).round() / 10.0,
            "has_surplus": gap >= 0.0,
        }));
    }

    let total_domains = series.len() as i64;
    let surplus_count = domain_charts.iter()
        .filter(|d| d["has_surplus"].as_bool().unwrap_or(false))
        .count() as i64;
    let deficit_count = total_domains - surplus_count;

    ctx.insert("domain_charts", &domain_charts);
    ctx.insert("summary", &json!({
        "total_domains": total_domains,
        "surplus_count": surplus_count,
        "deficit_count": deficit_count,
    }));

    render_page(&data, "analytics/supply_demand.html", &ctx)
}
