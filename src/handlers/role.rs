use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::{Identity};
use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;
use serde_json::json;

use std::collections::BTreeMap;
use std::sync::Arc;
use crate::{AppData, generate_basic_context, by_lang, level_weight, chart_json};
use crate::graphql::{get_role_by_id, get_role_matches, all_roles, all_organizations, get_team_by_id, get_people_by_name, create_role, update_role, assign_person_to_role, vacate_role, get_skill_by_id, create_requirement, update_requirement, get_person_by_id, update_work, create_role_offer};
use crate::security::{self, MinimumRole};
use super::org_tier::humanize;
use super::capability::CAPABILITY_LEVELS;

/// Rank enum values, kept in sync with the API schema.
pub const RANKS: [&str; 17] = [
    "PRIVATE", "CORPORAL", "MASTER_CORPORAL", "SERGEANT", "WARRANT_OFFICER",
    "MASTER_WARRANT_OFFICER", "CHIEF_WARRANT_OFFICER", "SECOND_LIEUTENANT",
    "LIEUTENANT", "CAPTAIN", "MAJOR", "LIEUTENANT_COLONEL", "COLONEL",
    "BRIGADIER_GENERAL", "MAJOR_GENERAL", "LIEUTENANT_GENERAL", "GENERAL",
];

/// MilitaryOccupation enum values, kept in sync with the API schema.
pub const MILITARY_OCCUPATIONS: [&str; 36] = [
    "INFANTRY", "ARMOURED", "ARTILLERY", "COMBAT_ENGINEERS", "SIGNALS",
    "INTELLIGENCE", "MILITARY_POLICE", "LOGISTICS_SUPPORT", "MEDICAL_TECHNICIAN",
    "COMMUNICATIONS", "ELECTRONICS", "VEHICLE_TECHNICIAN", "WEAPONS_TECHNICIAN",
    "SUPPLY_TECHNICIAN", "COOK_SUPPORT", "FINANCE_CLERK",
    "HUMAN_RESOURCES_ADMINISTRATOR", "MILITARY_FIREFIGHTER",
    "MATERIALS_MANAGEMENT", "GEOMATICS_TECHNICIAN", "MEDICAL_ASSISTANT",
    "DENTAL_ASSISTANT", "PHARMACY_TECHNICIAN", "CHAPLAIN", "LEGAL_OFFICER",
    "PILOT", "AIRCREW_SYSTEMS", "AIR_TRAFFIC_CONTROLLER", "WEATHER_TECHNICIAN",
    "IMAGE_TECHNICIAN", "MUSICIAN", "PHYSICAL_FITNESS_INSTRUCTOR", "CYBER",
    "SPECIAL_FORCES", "OFFICER", "OTHER",
];

/// OccupationalGroup enum values (civilian classifications), kept in sync
/// with the API schema.
pub const OCCUPATIONAL_GROUPS: [&str; 12] = [
    "ADMINISTRATIVE_SERVICES", "COMPUTER_SYSTEMS", "ECONOMICS_AND_SOCIAL_SCIENCE",
    "ENGINEERING", "EXECUTIVE", "FINANCIAL_MANAGEMENT", "HUMAN_RESOURCES",
    "INFORMATION_SERVICES", "PROGRAM_ADMINISTRATION", "RESEARCH",
    "TECHNICAL_SERVICES", "OTHER",
];

fn enum_options(values: &[&str]) -> serde_json::Value {
    json!(values
        .iter()
        .map(|value| json!({"value": value, "label": humanize(value)}))
        .collect::<Vec<serde_json::Value>>())
}

#[derive(Deserialize, Debug)]
pub struct RoleForm {
    pub csrf_token: String,
    pub organization_id: String,
    pub org_tier_id: String,
    pub team_id: String,
    pub title_en: String,
    pub title_fr: String,
    pub effort: f64,
    // Military role holders set these; civilian holders leave them blank.
    #[serde(default)]
    pub military_occupation: String,
    #[serde(default)]
    pub rank: String,
    // Civilian role holders set these; military holders leave them blank.
    #[serde(default)]
    pub occupational_group: String,
    #[serde(default)]
    pub occupational_level: String,
    pub start_date: String,
    // Optional: full name of the person to assign; blank creates a vacant role
    #[serde(default)]
    pub person_name: String,
}

#[derive(Deserialize, Debug)]
pub struct RoleStatusForm {
    pub csrf_token: String,
    // Checkbox: present ("true") when checked, absent otherwise
    #[serde(default)]
    pub active: Option<String>,
    pub start_date: String,
    #[serde(default)]
    pub end_date: String,
}

#[derive(Deserialize, Debug)]
pub struct EndRoleForm {
    pub csrf_token: String,
}

/// Default tuning for the vacant-role candidate matcher. `min_coverage` is the
/// fraction of requirements a "close" candidate must meet; `max_gap_per_req` is
/// the largest single-skill shortfall tolerated. `LIMIT` caps each tier.
const DEFAULT_MIN_COVERAGE: f64 = 0.5;
const DEFAULT_MAX_GAP_PER_REQ: i64 = 1;
const MATCH_LIMIT: i64 = 10;

/// Query params for the HTMX-driven match panel: the two sliders post their
/// values here so the panel re-renders without a full page load.
#[derive(Deserialize, Debug)]
pub struct MatchParams {
    #[serde(default)]
    pub min_coverage: Option<f64>,
    #[serde(default)]
    pub max_gap_per_req: Option<i64>,
}

#[derive(Deserialize, Debug)]
pub struct NewRoleParams {
    pub team: String,
    #[serde(default)]
    pub org_tier: String,
    #[serde(default)]
    pub organization: String,
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

fn parse_date(value: &str) -> Option<NaiveDateTime> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
}

fn role_from_form(form: &RoleForm) -> serde_json::Value {
    json!({
        "titleEnglish": form.title_en,
        "titleFrench": form.title_fr,
        "effort": form.effort,
        "militaryOccupation": form.military_occupation,
        "rank": form.rank,
        "occupationalGroup": form.occupational_group,
        "occupationalLevel": form.occupational_level,
        "startDate": form.start_date,
        "personName": form.person_name,
        "teamId": form.team_id,
        "orgTierId": form.org_tier_id,
        "organizationId": form.organization_id,
    })
}

#[get("/{lang}/role/{role_id}")]
pub async fn role_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req:HttpRequest) -> impl Responder {
    let (lang, role_id) = path_params.into_inner();

    let session = req.get_session();

    let mut ctx= generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_role_by_id(role_id.clone(), bearer.clone(), &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get role");

    // Requirement-match bars: compare each role requirement against the level
    // the incumbent actually holds in that domain (validated preferred, self
    // as fallback). Only built for an occupied role that has requirements.
    let role_rec = &r.role_by_id;
    if !role_rec.requirements.is_empty() {
        if let Some(person) = &role_rec.person {
            let mut held_by_domain: BTreeMap<String, i64> = BTreeMap::new();
            for cap in &person.capabilities {
                let domain = serde_json::to_value(&cap.domain)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default();
                let self_w = serde_json::to_value(&cap.self_identified_level)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .map(|s| level_weight(&s))
                    .unwrap_or(0);
                let val_w = cap.validated_level.as_ref()
                    .and_then(|l| serde_json::to_value(l).ok())
                    .and_then(|v| v.as_str().map(String::from))
                    .map(|s| level_weight(&s))
                    .unwrap_or(0);
                let held = if val_w > 0 { val_w } else { self_w };
                let e = held_by_domain.entry(domain).or_insert(0);
                if held > *e { *e = held; }
            }

            let labels: Vec<String> = role_rec.requirements.iter()
                .map(|req| req.name_en.clone())
                .collect();
            let required: Vec<i64> = role_rec.requirements.iter()
                .map(|req| serde_json::to_value(&req.required_level)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .map(|s| level_weight(&s))
                    .unwrap_or(0))
                .collect();
            let held: Vec<serde_json::Value> = role_rec.requirements.iter()
                .enumerate()
                .map(|(i, req)| {
                    let domain = serde_json::to_value(&req.domain)
                        .ok()
                        .and_then(|v| v.as_str().map(String::from))
                        .unwrap_or_default();
                    let held_w = *held_by_domain.get(&domain).unwrap_or(&0);
                    let meets = held_w >= required[i];
                    json!({
                        "value": held_w,
                        "itemStyle": {"color": if meets { "#198754" } else { "#dc3545" }},
                    })
                })
                .collect();

            let req_match = json!({
                "tooltip": {"trigger": "axis"},
                "legend": {"data": ["Required", "Held"], "bottom": 0},
                "grid": {"left": "3%", "right": "4%", "bottom": "14%", "containLabel": true},
                "xAxis": {"type": "category", "data": labels, "axisLabel": {"rotate": 20, "interval": 0}},
                "yAxis": {"type": "value", "max": 5, "name": "Level"},
                "series": [
                    {"name": "Required", "type": "bar", "data": required, "itemStyle": {"color": "#6c757d"}},
                    {"name": "Held", "type": "bar", "data": held}
                ]
            });
            ctx.insert("requirement_match", &chart_json(&req_match));
        }
    }

    // Vacant roles with requirements get the scored candidate matcher seeded
    // with default tuning. The panel itself can re-tune via HTMX (see
    // `role_matches`). Roles without requirements have nothing to match against.
    if role_rec.person.is_none() && !role_rec.requirements.is_empty() {
        ctx.insert("match_role_id", &role_id);
        match get_role_matches(
            role_id.clone(),
            DEFAULT_MIN_COVERAGE,
            DEFAULT_MAX_GAP_PER_REQ,
            MATCH_LIMIT,
            bearer,
            &data.api_url,
            Arc::clone(&data.client),
        ).await {
            Ok(resp) => {
                let m = resp.role_by_id.fuzzy_matches;
                ctx.insert("match_managed_full", &m.managed_full_matches);
                ctx.insert("match_managed_partial", &m.managed_partial_matches);
                ctx.insert("match_external_full", &m.external_full_matches);
                ctx.insert("match_external_partial", &m.external_partial_matches);
            },
            // Don't blow up the whole page if matching fails; the panel just
            // renders empty with the error noted in the template fallback.
            Err(e) => ctx.insert("match_error", &e.to_string()),
        }
        ctx.insert("min_coverage_pct", &((DEFAULT_MIN_COVERAGE * 100.0).round() as i64));
        ctx.insert("max_gap_per_req", &DEFAULT_MAX_GAP_PER_REQ);
    }

    ctx.insert("role_record", &r.role_by_id);

    let rendered = data.tmpl.render("role/role.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// HTMX target for the vacant-role candidate matcher. The min-coverage and
/// max-gap sliders fire `hx-get` here with their values; this re-runs the
/// scored matcher and returns just the `role/_matches.html` panel. The page
/// degrades gracefully without JS because the initial panel is server-rendered
/// on the role page itself.
#[get("/{lang}/role/{role_id}/matches")]
pub async fn role_matches(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    params: web::Query<MatchParams>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id) = path_params.into_inner();
    let session = req.get_session();

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match session.get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    // Clamp slider inputs to the ranges the UI offers so a hand-edited query
    // string can't push nonsense into the API.
    let min_coverage = params.min_coverage.unwrap_or(DEFAULT_MIN_COVERAGE).clamp(0.0, 1.0);
    let max_gap_per_req = params.max_gap_per_req.unwrap_or(DEFAULT_MAX_GAP_PER_REQ).clamp(0, 4);

    ctx.insert("match_role_id", &role_id);
    ctx.insert("min_coverage_pct", &((min_coverage * 100.0).round() as i64));
    ctx.insert("max_gap_per_req", &max_gap_per_req);

    match get_role_matches(
        role_id,
        min_coverage,
        max_gap_per_req,
        MATCH_LIMIT,
        bearer,
        &data.api_url,
        Arc::clone(&data.client),
    ).await {
        Ok(resp) => {
            let m = resp.role_by_id.fuzzy_matches;
            ctx.insert("match_managed_full", &m.managed_full_matches);
            ctx.insert("match_managed_partial", &m.managed_partial_matches);
            ctx.insert("match_external_full", &m.external_full_matches);
            ctx.insert("match_external_partial", &m.external_partial_matches);
        },
        Err(e) => ctx.insert("match_error", &e.to_string()),
    }

    let rendered = data.tmpl.render("role/_matches.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// Form to create a role in a team. Takes ?team=<uuid> plus
/// &org_tier=&organization= so the builder can re-render the right
/// branch. HTMX requests get the inline partial.
#[get("/{lang}/role/new")]
pub async fn create_role_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    params: web::Query<NewRoleParams>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let team = match get_team_by_id(params.team.clone(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.team_by_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("role_form", &json!({
        "titleEnglish": "", "titleFrench": "", "effort": 10.0,
        "militaryOccupation": "", "rank": "", "personName": "",
        "occupationalGroup": "", "occupationalLevel": "",
        "startDate": chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string(),
        "teamId": team.id,
        "orgTierId": if params.org_tier.is_empty() { team.organization_level.id.clone() } else { params.org_tier.clone() },
        "organizationId": if params.organization.is_empty() { team.organization.id.clone() } else { params.organization.clone() },
    }));
    ctx.insert("team", &team);
    ctx.insert("ranks", &enum_options(&RANKS));
    ctx.insert("military_occupations", &enum_options(&MILITARY_OCCUPATIONS));
    ctx.insert("occupational_groups", &enum_options(&OCCUPATIONAL_GROUPS));

    let template = if is_htmx(&req) {
        "org_chart/add_role_form.html"
    } else {
        "role/role_form.html"
    };

    let rendered = data.tmpl.render(template, &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/role/new")]
pub async fn create_role_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    form: web::Form<RoleForm>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/role/new?team={}", &lang, &form.team_id));
    }

    // Everything that can fail collects into form_error / flash and
    // re-renders the form with the submitted values preserved
    let mut form_error: Option<String> = None;

    // Resolve the optional assignee by full name. The API's personByName
    // does an ilike against family OR given name separately, so it can't
    // match a "Given Family" string directly. Search by the most
    // discriminating token (the last word) and then filter the candidates
    // for an exact full-name match.
    let mut person_id: Option<String> = None;
    let typed_name = form.person_name.trim().to_string();
    if !typed_name.is_empty() {
        let search_token = typed_name
            .split_whitespace()
            .last()
            .unwrap_or(&typed_name)
            .to_string();
        match get_people_by_name(search_token, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
            Ok(r) => {
                let matches = r.person_by_name;
                let exact: Vec<_> = matches
                    .iter()
                    .filter(|person| {
                        format!("{} {}", person.given_name, person.family_name)
                            .eq_ignore_ascii_case(&typed_name)
                    })
                    .collect();
                if exact.len() == 1 {
                    person_id = Some(exact[0].id.clone());
                } else if exact.len() > 1 {
                    form_error = Some(by_lang(
                        &lang,
                        "Several people share that exact name — assign the role from the person's page instead.",
                        "Plusieurs personnes portent exactement ce nom — affectez le rôle depuis la page de la personne.",
                    ).to_string());
                } else if matches.len() == 1 {
                    // Single candidate but the typed name wasn't an exact
                    // full-name match — accept it (handles a lone token)
                    person_id = Some(matches[0].id.clone());
                } else if matches.is_empty() {
                    form_error = Some(by_lang(
                        &lang,
                        "No person found with that name. Leave blank to create a vacant role.",
                        "Aucune personne trouvée avec ce nom. Laissez vide pour créer un rôle vacant.",
                    ).to_string());
                } else {
                    form_error = Some(by_lang(
                        &lang,
                        "Several people match that name — please use the full given and family name.",
                        "Plusieurs personnes correspondent à ce nom — veuillez utiliser le prénom et le nom complets.",
                    ).to_string());
                }
            },
            Err(e) => form_error = Some(e.to_string()),
        }
    }

    let start = parse_date(&form.start_date);
    if form_error.is_none() && start.is_none() {
        form_error = Some(by_lang(&lang, "Invalid start date.", "Date de début invalide.").to_string());
    }

    // Military and civilian classifications are mutually exclusive: a role is
    // one or the other, never both. The client form disables the opposite
    // group, but enforce it here too for non-JS / direct posts.
    let has_military = !form.rank.trim().is_empty() || !form.military_occupation.trim().is_empty();
    let has_civilian = !form.occupational_group.trim().is_empty() || !form.occupational_level.trim().is_empty();
    if form_error.is_none() && has_military && has_civilian {
        form_error = Some(by_lang(
            &lang,
            "A role must be either military or civilian, not both. Clear one classification.",
            "Un rôle doit être soit militaire, soit civil, pas les deux. Effacez une classification.",
        ).to_string());
    }

    if form_error.is_none() {
        // Military and civilian classifications are mutually exclusive and
        // all optional on the API: send Some only for the fields the form
        // actually filled in, leaving the rest null.
        let blank_to_none = |value: &str| -> Option<serde_json::Value> {
            let trimmed = value.trim();
            if trimmed.is_empty() { None } else { Some(json!(trimmed)) }
        };

        let new_role = create_role::NewRole {
            person_id,
            team_id: form.team_id.clone(),
            title_en: form.title_en.trim().to_string(),
            title_fr: form.title_fr.trim().to_string(),
            effort: form.effort,
            active: true,
            military_occupation: blank_to_none(&form.military_occupation)
                .map(|v| serde_json::from_value(v).expect("MilitaryOccupation deserialization is infallible")),
            rank: blank_to_none(&form.rank)
                .map(|v| serde_json::from_value(v).expect("Rank deserialization is infallible")),
            occupational_group: blank_to_none(&form.occupational_group)
                .map(|v| serde_json::from_value(v).expect("OccupationalGroup deserialization is infallible")),
            occupational_level: form.occupational_level.trim().parse::<i64>().ok(),
            start_datestamp: start.unwrap(),
            end_date: None,
            // Reporting line is managed separately (defaults to the team owner).
            reports_to: None,
        };

        match create_role(new_role, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
            Ok(response) => {
                if is_htmx(&req) {
                    // In the builder: re-render the tier node so the new
                    // role appears in its team
                    return super::org_chart::render_node_response(
                        &data, &session, id, &lang, &form.org_tier_id, &form.organization_id, &req,
                    ).await;
                }
                security::add_flash(
                    &session,
                    "success",
                    by_lang(&lang, "Role created.", "Rôle créé."),
                );
                return redirect_to(format!("/{}/role/{}", &lang, response.create_role.id));
            },
            Err(e) => form_error = Some(e.to_string()),
        }
    }

    // Error path: re-render with input preserved
    let error = form_error.expect("error path always has a message");
    if !is_htmx(&req) {
        security::add_flash(&session, "danger", &error);
    }

    let team = get_team_by_id(form.team_id.clone(), auth.bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .map(|r| serde_json::to_value(&r.team_by_id).unwrap_or(json!(null)))
        .unwrap_or(json!(null));

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("role_form", &role_from_form(&form));
    ctx.insert("team", &team);
    ctx.insert("ranks", &enum_options(&RANKS));
    ctx.insert("military_occupations", &enum_options(&MILITARY_OCCUPATIONS));
    ctx.insert("occupational_groups", &enum_options(&OCCUPATIONAL_GROUPS));

    let template = if is_htmx(&req) {
        ctx.insert("form_error", &error);
        "org_chart/add_role_form.html"
    } else {
        "role/role_form.html"
    };

    let rendered = data.tmpl.render(template, &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// The API only allows changing a role's active flag and dates — the
/// edit page is a status form, not a full edit (create a new role to
/// change titles or assignment, preserving history).
#[get("/{lang}/role/{role_id}/edit")]
pub async fn edit_role_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_role_by_id(role_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("role_record", &r.role_by_id);

    let rendered = data.tmpl.render("role/role_status_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/role/{role_id}/edit")]
pub async fn edit_role_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<RoleStatusForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/role/{}/edit", &lang, &role_id));
    }

    let role_data = update_role::RoleData {
        id: role_id.clone(),
        active: Some(form.active.is_some()),
        start_datestamp: parse_date(&form.start_date),
        end_date: parse_date(&form.end_date),
    };

    match update_role(role_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => {
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Role updated.", "Rôle mis à jour."),
            );
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
        },
    };

    redirect_to(format!("/{}/role/{}", &lang, &role_id))
}

#[get("/{lang}/role/{role_id}/end")]
pub async fn end_role_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_role_by_id(role_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("role_record", &r.role_by_id);

    let rendered = data.tmpl.render("role/role_end.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/role/{role_id}/end")]
pub async fn end_role_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<EndRoleForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/role/{}", &lang, &role_id));
    }

    let role_data = update_role::RoleData {
        id: role_id.clone(),
        active: Some(false),
        start_datestamp: None,
        end_date: Some(chrono::Utc::now().naive_utc()),
    };

    match update_role(role_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => {
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Role ended.", "Rôle terminé."),
            );
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
        },
    };

    redirect_to(format!("/{}/role/{}", &lang, &role_id))
}

#[derive(Deserialize, Debug)]
pub struct AssignRoleForm {
    pub csrf_token: String,
    pub person_id: String,
    #[serde(default)]
    pub reassign_work_to: String,
}

#[derive(Deserialize, Debug)]
pub struct TransferPreviewParams {
    pub person_id: String,
}

#[derive(Deserialize, Debug)]
pub struct OfferRoleForm {
    pub csrf_token: String,
    pub person_id: String,
    #[serde(default)]
    pub message: String,
}

/// Render the transfer-confirmation modal (HTMX) or a full confirmation page
/// (plain navigation fallback) for assigning `person_id` to this role. Shows
/// the role(s) the person currently holds and the work attached to them so an
/// operator can confirm — and optionally reassign that work to the new role.
#[get("/{lang}/role/{role_id}/transfer-preview")]
pub async fn transfer_preview(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    params: web::Query<TransferPreviewParams>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let role = match get_role_by_id(role_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.role_by_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/role/{}", &lang, &role_id));
        },
    };

    let person = match get_person_by_id(params.person_id.clone(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.person_by_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/role/{}", &lang, &role_id));
        },
    };

    let target_title = if lang == "fr" { role.title_french.clone() } else { role.title_english.clone() };
    let has_work = person.active_roles.iter().any(|r| !r.work.is_empty());

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("person", &person);
    ctx.insert("target_role_id", &role_id);
    ctx.insert("target_role_title", &target_title);
    ctx.insert("has_work", &has_work);
    ctx.insert("cancel_url", &format!("/{}/role/{}", &lang, &role_id));

    let template = if is_htmx(&req) {
        "role/_transfer_confirm.html"
    } else {
        "role/transfer_confirm.html"
    };

    let rendered = data.tmpl.render(template, &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// Assign a person to a vacant role. Driven by the "Assign" buttons on the
/// role detail (potential matches), person detail (potential job matches),
/// and vacancy pages — each posts the chosen person_id here.
#[post("/{lang}/role/{role_id}/assign")]
pub async fn assign_role_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<AssignRoleForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/role/{}", &lang, &role_id));
    }

    if form.person_id.trim().is_empty() {
        security::add_flash(&session, "danger", by_lang(&lang, "No person selected.", "Aucune personne sélectionnée."));
        return redirect_to(format!("/{}/role/{}", &lang, &role_id));
    }

    let reassign_target = form.reassign_work_to.trim().to_string();
    let mut work_ids_to_reassign: Vec<String> = Vec::new();

    if !reassign_target.is_empty() {
        if let Ok(person) = get_person_by_id(form.person_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
            for ar in &person.person_by_id.active_roles {
                for w in &ar.work {
                    work_ids_to_reassign.push(w.id.clone());
                }
            }
        }
    }

    match assign_person_to_role(form.person_id.clone(), role_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => {
            let mut reassign_count = 0;
            for work_id in &work_ids_to_reassign {
                let work_data = update_work::WorkData {
                    id: work_id.clone(),
                    task_id: None,
                    role_id: Some(reassign_target.clone()),
                    skill_id: None,
                    work_description: None,
                    url: None,
                    domain: None,
                    capability_level: None,
                    effort: None,
                    work_status: None,
                    priority: None,
                    // Bulk reassignment only moves the role; dates and blocked
                    // context are left untouched.
                    due_date: None,
                    blocked_reason: None,
                    blocked_on_role_id: None,
                };
                if update_work(work_data, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await.is_ok() {
                    reassign_count += 1;
                }
            }

            if reassign_count > 0 {
                security::add_flash(&session, "success", by_lang(
                    &lang,
                    &format!("Person assigned to role. {} work item(s) reassigned. Any role they previously held has been vacated and recorded in their history.", reassign_count),
                    &format!("Personne affectée au rôle. {} élément(s) de travail réaffecté(s). Tout rôle qu'elle occupait auparavant a été libéré et inscrit dans son historique.", reassign_count),
                ));
            } else {
                security::add_flash(&session, "success", by_lang(
                    &lang,
                    "Person assigned to role. Any role they previously held has been vacated and recorded in their history.",
                    "Personne affectée au rôle. Tout rôle qu'elle occupait auparavant a été libéré et inscrit dans son historique.",
                ));
            }
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
        },
    };

    redirect_to(format!("/{}/role/{}", &lang, &role_id))
}

/// Remove the person from a role, leaving it vacant.
#[post("/{lang}/role/{role_id}/vacate")]
pub async fn vacate_role_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<EndRoleForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/role/{}", &lang, &role_id));
    }

    match vacate_role(role_id.clone(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => {
            security::add_flash(&session, "success", by_lang(&lang, "Role vacated.", "Rôle libéré."));
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
        },
    };

    redirect_to(format!("/{}/role/{}", &lang, &role_id))
}

/// Offer this (vacant) role to a candidate who sits outside the caller's managed
/// area. Instead of an immediate transfer, this creates a RoleOffer the
/// candidate's current manager must accept. The matching panel shows this action
/// only for out-of-area candidates.
#[post("/{lang}/role/{role_id}/offer")]
pub async fn offer_role_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<OfferRoleForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/role/{}", &lang, &role_id));
    }

    if form.person_id.trim().is_empty() {
        security::add_flash(&session, "danger", by_lang(&lang, "No person selected.", "Aucune personne sélectionnée."));
        return redirect_to(format!("/{}/role/{}", &lang, &role_id));
    }

    let message = match form.message.trim() {
        "" => None,
        m => Some(m.to_string()),
    };

    match create_role_offer(role_id.clone(), form.person_id.clone(), message, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(resp) => {
            security::add_flash(&session, "success", by_lang(&lang,
                "Offer sent — the candidate's manager will be asked to approve the transfer.",
                "Offre envoyée — le gestionnaire du candidat sera invité à approuver le transfert."));

            // Notify the approving manager (no-op unless notifications are
            // enabled — see notifications::send_offer_email).
            let offer = resp.create_role_offer;
            let to = offer.approver_role.as_ref().and_then(|r| r.person.as_ref()).map(|p| p.email.clone());
            let html = format!(
                "<p>{} {} has been offered the role “{}”. Review the offer in your manager panel.</p>",
                offer.person.given_name, offer.person.family_name, offer.role.title_english,
            );
            crate::notifications::send_offer_email(to.as_deref(), "New transfer offer to review", &html).await;
        },
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    }

    redirect_to(format!("/{}/role/{}", &lang, &role_id))
}

#[derive(Deserialize, Debug)]
pub struct RequirementForm {
    pub csrf_token: String,
    pub skill_id: String,
    pub required_level: String,
}

#[derive(Deserialize, Debug)]
pub struct RequirementRetireForm {
    pub csrf_token: String,
}

#[derive(Deserialize, Debug)]
pub struct RequirementEditForm {
    pub csrf_token: String,
    pub required_level: String,
}

fn level_options() -> serde_json::Value {
    json!(CAPABILITY_LEVELS.iter().map(|l| json!({"value": l, "label": humanize(l)})).collect::<Vec<serde_json::Value>>())
}

#[get("/{lang}/role/{role_id}/requirement/new")]
pub async fn create_requirement_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let (skill_domains, skill_groups) = super::skill::skill_picker_data(&data, auth.bearer).await;

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("role_id", &role_id);
    ctx.insert("skill_domains", &skill_domains);
    ctx.insert("skill_groups", &skill_groups);
    ctx.insert("capability_levels", &level_options());

    let rendered = data.tmpl.render("role/requirement_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/role/{role_id}/requirement/new")]
pub async fn create_requirement_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<RequirementForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/role/{}/requirement/new", &lang, &role_id));
    }

    // The chosen skill supplies the requirement's name and domain
    let skill = match get_skill_by_id(form.skill_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.skill_by_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/role/{}/requirement/new", &lang, &role_id));
        },
    };

    let new_requirement = create_requirement::NewRequirement {
        name_en: skill.name_en.clone(),
        name_fr: skill.name_fr.clone(),
        domain: serde_json::from_value(json!(skill.domain)).expect("SkillDomain deserialization is infallible"),
        role_id: role_id.clone(),
        skill_id: form.skill_id.clone(),
        required_level: serde_json::from_value(json!(form.required_level)).expect("CapabilityLevel deserialization is infallible"),
    };

    match create_requirement(new_requirement, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Requirement added.", "Exigence ajoutée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/role/{}", &lang, &role_id))
}

#[post("/{lang}/role/{role_id}/requirement/{requirement_id}/retire")]
pub async fn retire_requirement_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String, String)>,
    form: web::Form<RequirementRetireForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id, requirement_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/role/{}", &lang, &role_id));
    }

    let requirement_data = update_requirement::RequirementData {
        id: requirement_id,
        name_en: None,
        name_fr: None,
        domain: None,
        required_level: None,
        retired_at: Some(chrono::Utc::now().naive_utc()),
    };

    match update_requirement(requirement_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Requirement retired.", "Exigence retirée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/role/{}", &lang, &role_id))
}

#[get("/{lang}/role/{role_id}/requirement/{requirement_id}/edit")]
pub async fn edit_requirement_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id, requirement_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let role = match get_role_by_id(role_id.clone(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.role_by_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/role/{}", &lang, &role_id));
        },
    };

    let requirement = match role.requirements.iter().find(|r| r.id == requirement_id) {
        Some(r) => r,
        None => {
            security::add_flash(&session, "danger", by_lang(&lang, "Requirement not found.", "Exigence introuvable."));
            return redirect_to(format!("/{}/role/{}", &lang, &role_id));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &true);
    ctx.insert("role_id", &role_id);
    ctx.insert("requirement", requirement);
    ctx.insert("capability_levels", &level_options());

    let rendered = data.tmpl.render("role/requirement_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/role/{role_id}/requirement/{requirement_id}/edit")]
pub async fn edit_requirement_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String, String)>,
    form: web::Form<RequirementEditForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, role_id, requirement_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/role/{}/requirement/{}/edit", &lang, &role_id, &requirement_id));
    }

    let requirement_data = update_requirement::RequirementData {
        id: requirement_id.clone(),
        name_en: None,
        name_fr: None,
        domain: None,
        required_level: Some(
            serde_json::from_value(json!(form.required_level))
                .expect("CapabilityLevel deserialization is infallible"),
        ),
        retired_at: None,
    };

    match update_requirement(requirement_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Requirement updated.", "Exigence mise à jour.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/role/{}", &lang, &role_id))
}

#[derive(Deserialize, Debug)]
pub struct RoleIndexParams {
    #[serde(default)]
    pub q: String,
    /// Organization UUID to filter by; empty means all organizations.
    #[serde(default)]
    pub org: String,
    /// "filled" | "vacant" | "" (all). Whether the role has an incumbent.
    #[serde(default)]
    pub status: String,
}

#[get("/{lang}/roles")]
pub async fn role_index(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    params: web::Query<RoleIndexParams>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let query = params.q.trim().to_lowercase();
    let selected_org = params.org.trim().to_string();
    let selected_status = params.status.trim();
    // allRoles is already active-only on the API side
    let roles = all_roles(bearer.clone(), &data.api_url, Arc::clone(&data.client)).await
        .map(|r| r.all_roles)
        .unwrap_or_default();

    let matched: Vec<_> = roles.iter()
        .filter(|r| query.is_empty()
            || r.title_english.to_lowercase().contains(&query)
            || r.title_french.to_lowercase().contains(&query)
            || r.person.as_ref().map_or(false, |p| format!("{} {}", p.given_name, p.family_name).to_lowercase().contains(&query)))
        .filter(|r| selected_org.is_empty() || r.team.organization.id == selected_org)
        .filter(|r| match selected_status {
            "filled" => r.person.is_some(),
            "vacant" => r.person.is_none(),
            _ => true,
        })
        .collect();
    let total = matched.len();
    let visible: Vec<_> = matched.into_iter().take(super::person::INDEX_PAGE_CAP).collect();

    // Organization filter options, active orgs only, sorted by name.
    let mut organizations = all_organizations(bearer, &data.api_url, Arc::clone(&data.client)).await
        .map(|r| r.all_organizations)
        .unwrap_or_default();
    organizations.retain(|o| o.retired_at.is_none());
    organizations.sort_by(|a, b| a.name_en.to_lowercase().cmp(&b.name_en.to_lowercase()));

    ctx.insert("roles", &visible);
    ctx.insert("total", &total);
    ctx.insert("truncated", &(total > super::person::INDEX_PAGE_CAP));
    ctx.insert("q", &params.q);
    ctx.insert("organizations", &organizations);
    ctx.insert("selected_org", &selected_org);
    ctx.insert("selected_status", &selected_status);

    let template = if is_htmx(&req) { "role/role_list.html" } else { "role/role_index.html" };
    let rendered = data.tmpl.render(template, &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}
