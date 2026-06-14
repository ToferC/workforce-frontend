use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::{Identity};
use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;
use serde_json::json;

use std::sync::Arc;
use crate::{AppData, generate_basic_context, by_lang};
use crate::graphql::{get_role_by_id, all_roles, get_team_by_id, get_people_by_name, create_role, update_role, all_skills, get_skill_by_id, create_requirement, update_requirement};
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
    pub military_occupation: String,
    pub rank: String,
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

    let r = get_role_by_id(role_id, bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get role");

    ctx.insert("role_record", &r.role_by_id);

    let rendered = data.tmpl.render("role/role.html", &ctx).unwrap();
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
        "titleEnglish": "", "titleFrench": "", "effort": 1.0,
        "militaryOccupation": "", "rank": "", "personName": "",
        "startDate": chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string(),
        "teamId": team.id,
        "orgTierId": if params.org_tier.is_empty() { team.organization_level.id.clone() } else { params.org_tier.clone() },
        "organizationId": if params.organization.is_empty() { team.organization.id.clone() } else { params.organization.clone() },
    }));
    ctx.insert("team", &team);
    ctx.insert("ranks", &enum_options(&RANKS));
    ctx.insert("military_occupations", &enum_options(&MILITARY_OCCUPATIONS));

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

    if form_error.is_none() {
        let new_role = create_role::NewRole {
            person_id,
            team_id: form.team_id.clone(),
            title_en: form.title_en.trim().to_string(),
            title_fr: form.title_fr.trim().to_string(),
            effort: form.effort,
            active: true,
            military_occupation: serde_json::from_value(json!(form.military_occupation))
                .expect("MilitaryOccupation deserialization is infallible"),
            rank: serde_json::from_value(json!(form.rank))
                .expect("Rank deserialization is infallible"),
            start_datestamp: start.unwrap(),
            end_date: None,
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
pub struct RequirementForm {
    pub csrf_token: String,
    pub skill_id: String,
    pub required_level: String,
}

#[derive(Deserialize, Debug)]
pub struct RequirementRetireForm {
    pub csrf_token: String,
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

    let skills = all_skills(auth.bearer, &data.api_url, Arc::clone(&data.client)).await
        .map(|r| json!(r.skills.iter().map(|s| json!({"value": s.id, "label": s.name_en})).collect::<Vec<_>>()))
        .unwrap_or_else(|_| json!([]));

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("role_id", &role_id);
    ctx.insert("skill_options", &skills);
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

#[derive(Deserialize, Debug)]
pub struct RoleIndexParams {
    #[serde(default)]
    pub q: String,
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
    // allRoles is already active-only on the API side
    let roles = all_roles(bearer, &data.api_url, Arc::clone(&data.client)).await
        .map(|r| r.all_roles)
        .unwrap_or_default();

    let matched: Vec<_> = roles.iter()
        .filter(|r| query.is_empty()
            || r.title_english.to_lowercase().contains(&query)
            || r.title_french.to_lowercase().contains(&query)
            || r.person.as_ref().map_or(false, |p| format!("{} {}", p.given_name, p.family_name).to_lowercase().contains(&query)))
        .collect();
    let total = matched.len();
    let visible: Vec<_> = matched.into_iter().take(super::person::INDEX_PAGE_CAP).collect();

    ctx.insert("roles", &visible);
    ctx.insert("total", &total);
    ctx.insert("truncated", &(total > super::person::INDEX_PAGE_CAP));
    ctx.insert("q", &params.q);

    let template = if is_htmx(&req) { "role/role_list.html" } else { "role/role_index.html" };
    let rendered = data.tmpl.render(template, &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}
