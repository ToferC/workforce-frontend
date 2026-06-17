use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use actix_identity::Identity;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::{AppData, generate_basic_context, status_color, chart_json};
use crate::graphql::{all_work, vacant_roles, analytics_people, analytics_roles, delivery_treemap, analytics_mobility};
use crate::security::{self, MinimumRole};

/// All 16 SkillDomain variants in canonical display order: (key, short label).
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

use crate::level_weight;

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

    let people_list = analytics_people(auth.bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .map(|r| r.all_people)
        .unwrap_or_default();

    let active_people: Vec<_> = people_list.iter()
        .filter(|p| p.retired_at.is_none())
        .collect();

    // team_name → domain_key → weighted depth score (sum of level weights per person)
    let mut depth_map: BTreeMap<String, BTreeMap<String, i64>> = BTreeMap::new();
    let mut team_ids: BTreeMap<String, String> = BTreeMap::new();

    for person in &active_people {
        if let Some(role) = person.active_roles.first() {
            let team_name = role.team.name_english.clone();
            team_ids.entry(team_name.clone()).or_insert_with(|| role.team.id.to_string());

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
                    *depth_map
                        .entry(team_name.clone())
                        .or_default()
                        .entry(domain)
                        .or_insert(0) += level_weight(&level_str);
                }
            }
        }
    }

    // Only include domains where at least one team has non-zero depth
    let active_domains: Vec<(&str, &str)> = SKILL_DOMAINS.iter()
        .filter(|(key, _)| depth_map.values().any(|m| m.get(*key).copied().unwrap_or(0) > 0))
        .copied()
        .collect();

    let domain_labels: Vec<&str> = active_domains.iter().map(|(_, label)| *label).collect();
    let team_names: Vec<&str> = depth_map.keys().map(String::as_str).collect();

    let max_depth: i64 = depth_map.values()
        .flat_map(|m| m.values())
        .copied()
        .max()
        .unwrap_or(1);

    // ECharts heatmap series data: [domain_idx, team_idx, depth]
    let mut heatmap_data: Vec<serde_json::Value> = Vec::new();
    for (t_idx, team_name) in team_names.iter().enumerate() {
        for (d_idx, (domain_key, _)) in active_domains.iter().enumerate() {
            let depth = depth_map.get(*team_name)
                .and_then(|m| m.get(*domain_key))
                .copied()
                .unwrap_or(0);
            heatmap_data.push(json!([d_idx, t_idx, depth]));
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

    // CSS fallback table — opacity pre-computed as float for Tera
    let table_rows: Vec<serde_json::Value> = depth_map.iter().map(|(team, domains)| {
        let team_id = team_ids.get(team).cloned().unwrap_or_default();
        let cells: Vec<serde_json::Value> = active_domains.iter().map(|(key, _)| {
            let depth = domains.get(*key).copied().unwrap_or(0);
            let opacity = if max_depth > 0 { depth as f64 / max_depth as f64 } else { 0.0 };
            json!({ "depth": depth, "opacity": opacity })
        }).collect();
        json!({ "team": team, "team_id": team_id, "cells": cells })
    }).collect();

    // Domain strength totals for the ranking sidebar
    let mut domain_totals: Vec<serde_json::Value> = active_domains.iter().map(|(key, label)| {
        let total: i64 = depth_map.values()
            .map(|m| m.get(*key).copied().unwrap_or(0))
            .sum();
        json!({ "domain": label, "key": key, "total": total })
    }).collect();
    domain_totals.sort_by(|a, b| {
        b["total"].as_i64().unwrap_or(0).cmp(&a["total"].as_i64().unwrap_or(0))
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
        "max_depth": max_depth,
    }));

    let rendered = data.tmpl.render("analytics/coverage.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
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

    // Build the nested treemap data (Product → Task → Work), pre-computing
    // every node's value so ECharts sizes rectangles correctly.
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
                }));
            }

            // Tasks with no work still get a slot sized by their own effort
            if task_value == 0 {
                task_value = std::cmp::max(task.effort, 1);
            }
            product_value += task_value;

            task_nodes.push(json!({
                "name": task.title,
                "value": task_value,
                "children": work_nodes,
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

        // Skip empty products in the treemap to keep it legible
        if !task_nodes.is_empty() {
            tree.push(json!({
                "name": product.name_en,
                "value": product_value,
                "children": task_nodes,
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

    // Status legend in canonical order
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

    let rendered = data.tmpl.render("analytics/delivery.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
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

    let people = analytics_mobility(auth.bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .map(|r| r.all_people)
        .unwrap_or_default();

    // Bipartite "previous team (was) → current team (now)" transitions.
    // For each person we take the team of their most recent prior role and
    // the team of their current role; a move is recorded when they differ.
    // The (was)/(now) split keeps the sankey acyclic by construction.
    let mut links: BTreeMap<(String, String), i64> = BTreeMap::new();
    let mut movers: i64 = 0;
    let mut teams_seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for person in &people {
        // Current team: the open (current) tenure's role team.
        let dest = person.role_assignments.iter()
            .find(|a| a.is_current)
            .map(|a| a.role.team.name_english.clone());

        // Previous team: the most recently closed tenure's role team.
        let origin = person.role_assignments.iter()
            .filter(|a| !a.is_current)
            .max_by(|a, b| a.end_date.cmp(&b.end_date))
            .map(|a| a.role.team.name_english.clone());

        if let (Some(o), Some(d)) = (origin, dest) {
            if o != d {
                teams_seen.insert(o.clone());
                teams_seen.insert(d.clone());
                *links.entry((o, d)).or_insert(0) += 1;
                movers += 1;
            }
        }
    }

    // Sankey nodes: origins labelled "(was)", destinations "(now)".
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

    let total_moves: i64 = links.values().sum();

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

    // Fallback / detail table, sorted by volume
    let mut table_rows: Vec<serde_json::Value> = links.iter()
        .map(|((o, d), count)| json!({"from": o, "to": d, "count": count}))
        .collect();
    table_rows.sort_by(|a, b| {
        b["count"].as_i64().unwrap_or(0).cmp(&a["count"].as_i64().unwrap_or(0))
    });

    ctx.insert("chart_option", &chart_json(&chart_option));
    ctx.insert("table_rows", &table_rows);
    ctx.insert("has_moves", &(!links.is_empty()));
    ctx.insert("summary", &json!({
        "total_moves": total_moves,
        "movers": movers,
        "teams_involved": teams_seen.len() as i64,
        "total_people": people.len() as i64,
    }));

    let rendered = data.tmpl.render("analytics/mobility.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}
