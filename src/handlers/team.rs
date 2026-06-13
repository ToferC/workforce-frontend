use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::{Identity};
use serde::Deserialize;
use serde_json::json;

use std::sync::Arc;
use crate::{AppData, generate_basic_context, by_lang};
use crate::graphql::{get_team_by_id, create_team, update_team, create_team_ownership, get_team_ownership_by_team_id, update_team_ownership};
use crate::security::{self, MinimumRole};
use super::org_tier::{parent_tier_options, skill_domain_options, OwnerForm};
use super::person::resolve_person_by_name;

#[derive(Deserialize, Debug)]
pub struct TeamForm {
    pub csrf_token: String,
    pub organization_id: String,
    pub org_tier_id: String,
    pub name_en: String,
    pub name_fr: String,
    pub description_en: String,
    pub description_fr: String,
    // Optional on edit (blank = keep current); Team doesn't expose its
    // current domain so the edit form can't pre-select it
    #[serde(default)]
    pub primary_domain: String,
}

#[derive(Deserialize, Debug)]
pub struct RetireForm {
    pub csrf_token: String,
}

#[derive(Deserialize, Debug)]
pub struct NewTeamParams {
    pub organization: String,
    #[serde(default)]
    pub org_tier: String,
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

fn team_from_form(form: &TeamForm, id: Option<&str>) -> serde_json::Value {
    json!({
        "id": id,
        "nameEnglish": form.name_en,
        "nameFrench": form.name_fr,
        "descriptionEnglish": form.description_en,
        "descriptionFrench": form.description_fr,
        "organization": {"id": form.organization_id},
        "organizationLevel": {"id": form.org_tier_id},
    })
}

#[get("/{lang}/team/{team_id}")]
pub async fn team_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req:HttpRequest) -> impl Responder {
    let (lang, team_id) = path_params.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_team_by_id(team_id, bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get team");

    ctx.insert("team", &r.team_by_id);

    let rendered = data.tmpl.render("team/team.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// Form to create a team. Takes ?organization=<uuid> and optionally
/// &org_tier=<uuid> so the org chart builder can pre-select the tier.
/// HTMX requests get the inline partial for the builder.
#[get("/{lang}/team/new")]
pub async fn create_team_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    params: web::Query<NewTeamParams>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let tier_options = match parent_tier_options(&params.organization, None, &auth.bearer, &data).await {
        Ok(options) => options,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/organization/{}", &lang, &params.organization));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &false);
    ctx.insert("team", &json!({
        "nameEnglish": "", "nameFrench": "", "descriptionEnglish": "", "descriptionFrench": "",
        "organization": {"id": params.organization},
        "organizationLevel": {"id": params.org_tier},
    }));
    ctx.insert("org_tier_options", &tier_options);
    ctx.insert("skill_domains", &skill_domain_options());

    let template = if is_htmx(&req) {
        "org_chart/add_team_form.html"
    } else {
        "team/team_form.html"
    };

    let rendered = data.tmpl.render(template, &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/team/new")]
pub async fn create_team_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    form: web::Form<TeamForm>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/team/new?organization={}", &lang, &form.organization_id));
    }

    let new_team = create_team::NewTeam {
        name_en: form.name_en.trim().to_string(),
        name_fr: form.name_fr.trim().to_string(),
        organization_id: form.organization_id.clone(),
        org_tier_id: form.org_tier_id.clone(),
        primary_domain: serde_json::from_value(json!(form.primary_domain))
            .expect("SkillDomain deserialization is infallible"),
        description_en: form.description_en.trim().to_string(),
        description_fr: form.description_fr.trim().to_string(),
    };

    match create_team(new_team, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            if is_htmx(&req) {
                // In the builder: re-render the tier node so the new team appears
                return super::org_chart::render_node_response(
                    &data, &session, id, &lang, &form.org_tier_id, &form.organization_id, &req,
                ).await;
            }
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Team created.", "Équipe créée."),
            );
            redirect_to(format!("/{}/team/{}", &lang, response.create_team.id))
        },
        Err(e) => {
            // Flash renders only on full pages; the inline partial shows
            // the error itself via form_error. Queue the flash before
            // generate_basic_context drains the queue.
            if !is_htmx(&req) {
                security::add_flash(&session, "danger", &e.to_string());
            }

            let tier_options = parent_tier_options(&form.organization_id, None, &auth.bearer, &data)
                .await
                .unwrap_or_else(|_| json!([]));

            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &false);
            ctx.insert("team", &team_from_form(&form, None));
            ctx.insert("org_tier_options", &tier_options);
            ctx.insert("skill_domains", &skill_domain_options());

            let template = if is_htmx(&req) {
                ctx.insert("form_error", &e.to_string());
                "org_chart/add_team_form.html"
            } else {
                "team/team_form.html"
            };

            let rendered = data.tmpl.render(template, &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}

#[get("/{lang}/team/{team_id}/edit")]
pub async fn edit_team_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, team_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_team_by_id(team_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &true);
    ctx.insert("team", &r.team_by_id);
    ctx.insert("skill_domains", &skill_domain_options());

    let rendered = data.tmpl.render("team/team_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/team/{team_id}/edit")]
pub async fn edit_team_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<TeamForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, team_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/team/{}/edit", &lang, &team_id));
    }

    let team_data = update_team::TeamData {
        id: team_id.clone(),
        name_en: Some(form.name_en.trim().to_string()),
        name_fr: Some(form.name_fr.trim().to_string()),
        // Team doesn't expose its current domain, so blank means unchanged
        primary_domain: if form.primary_domain.is_empty() {
            None
        } else {
            Some(serde_json::from_value(json!(form.primary_domain))
                .expect("SkillDomain deserialization is infallible"))
        },
        description_en: Some(form.description_en.trim().to_string()),
        description_fr: Some(form.description_fr.trim().to_string()),
        retired_at: None,
    };

    match update_team(team_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Team updated.", "Équipe mise à jour."),
            );
            redirect_to(format!("/{}/team/{}", &lang, response.update_team.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());

            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &true);
            ctx.insert("team", &team_from_form(&form, Some(&team_id)));
            ctx.insert("skill_domains", &skill_domain_options());

            let rendered = data.tmpl.render("team/team_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
    }
}

#[get("/{lang}/team/{team_id}/retire")]
pub async fn retire_team_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, team_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_team_by_id(team_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("team", &r.team_by_id);

    let rendered = data.tmpl.render("team/team_retire.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/team/{team_id}/retire")]
pub async fn retire_team_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<RetireForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, team_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/team/{}", &lang, &team_id));
    }

    let team_data = update_team::TeamData {
        id: team_id.clone(),
        name_en: None,
        name_fr: None,
        primary_domain: None,
        description_en: None,
        description_fr: None,
        retired_at: Some(chrono::Utc::now().naive_utc()),
    };

    match update_team(team_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => {
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Team retired.", "Équipe retirée."),
            );
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
        },
    };

    redirect_to(format!("/{}/team/{}", &lang, &team_id))
}

#[get("/{lang}/team/{team_id}/owner")]
pub async fn assign_team_owner_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, team_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_team_by_id(team_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("team", &r.team_by_id);

    let rendered = data.tmpl.render("team/assign_owner.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/team/{team_id}/owner")]
pub async fn assign_team_owner_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<OwnerForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, team_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/team/{}/owner", &lang, &team_id));
    }

    match resolve_person_by_name(&form.person_name, &auth.bearer, &lang, &data).await {
        Ok(Some(person_id)) => {
            // Reassign if the team already has an ownership record;
            // otherwise create one.
            let existing = get_team_ownership_by_team_id(team_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await.ok();
            let result = if let Some(existing) = existing {
                update_team_ownership(update_team_ownership::TeamOwnershipData {
                    id: existing.team_ownership_by_team_id.id,
                    person_id: Some(person_id),
                    team_id: None,
                    start_datestamp: None,
                    end_date: None,
                }, auth.bearer, &data.api_url, Arc::clone(&data.client)).await.map(|_| ())
            } else {
                create_team_ownership(create_team_ownership::NewTeamOwnership {
                    person_id,
                    team_id: team_id.clone(),
                    start_datestamp: chrono::Utc::now().naive_utc(),
                    end_date: None,
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

    redirect_to(format!("/{}/team/{}", &lang, &team_id))
}
