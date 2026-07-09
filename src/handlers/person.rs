use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::{Identity};
use serde::Deserialize;
use serde_json::json;

use std::collections::BTreeMap;
use std::sync::Arc;
use crate::{AppData, generate_basic_context, by_lang, level_weight, domain_short_label, chart_json};
use crate::graphql::{get_people_by_name, get_person_by_id, get_user_by_email, get_me, create_person, update_person, all_organizations, all_people, create_affiliation, update_affiliation, create_language_data, restore_person};
use crate::security::{self, MinimumRole};
use super::org_tier::humanize;
use super::utility::{redirect_to, csrf_failure_flash, is_htmx, render_confirm, render_page, session_bearer};

/// PersonnelType enum values, kept in sync with the API schema.
pub const PERSONNEL_TYPES: [&str; 5] = ["MILITARY", "CIVILIAN", "CONTRACTOR", "STUDENT", "OTHER"];

pub fn personnel_type_options() -> serde_json::Value {
    json!(PERSONNEL_TYPES.iter().map(|t| json!({"value": t, "label": humanize(t)})).collect::<Vec<serde_json::Value>>())
}

#[derive(Deserialize, Debug)]
pub struct PersonForm {
    pub csrf_token: String,
    // Create only: the email of the user account this person links to
    #[serde(default)]
    pub user_email: String,
    pub family_name: String,
    pub given_name: String,
    pub email: String,
    pub phone: String,
    pub work_address: String,
    pub city: String,
    pub province: String,
    pub postal_code: String,
    pub country: String,
    pub organization_id: String,
    #[serde(default)]
    pub peoplesoft_id: String,
    #[serde(default)]
    pub orcid_id: String,
    pub personnel_type: String,
}

#[derive(Deserialize, Debug)]
pub struct RetireForm {
    pub csrf_token: String,
}



pub async fn organization_options(bearer: &str, data: &AppData) -> serde_json::Value {
    match all_organizations(bearer.to_string(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => json!(r.all_organizations
            .iter()
            .map(|org| json!({"value": org.id, "label": org.name_en}))
            .collect::<Vec<serde_json::Value>>()),
        Err(_) => json!([]),
    }
}

/// Resolve a typed "Given Family" name to a person id. The API's
/// personByName does ilike on family OR given name separately, so it
/// can't match a concatenated name; search the last token and filter for
/// an exact full-name match. Returns Ok(None) for blank input,
/// Ok(Some(id)) when resolved, Err(message) when not found / ambiguous.
pub async fn resolve_person_by_name(name: &str, bearer: &str, lang: &str, data: &AppData) -> Result<Option<String>, String> {
    let typed = name.trim().to_string();
    if typed.is_empty() {
        return Ok(None);
    }
    let token = typed.split_whitespace().last().unwrap_or(&typed).to_string();
    match get_people_by_name(token, bearer.to_string(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => {
            let matches = r.person_by_name;
            let exact: Vec<_> = matches
                .iter()
                .filter(|p| format!("{} {}", p.given_name, p.family_name).eq_ignore_ascii_case(&typed))
                .collect();
            if exact.len() == 1 {
                Ok(Some(exact[0].id.clone()))
            } else if exact.len() > 1 {
                Err(by_lang(lang,
                    "Several people share that exact name — assign from the person's page instead.",
                    "Plusieurs personnes portent exactement ce nom — affectez depuis la page de la personne.").to_string())
            } else if matches.len() == 1 {
                Ok(Some(matches[0].id.clone()))
            } else if matches.is_empty() {
                Err(by_lang(lang, "No person found with that name.", "Aucune personne trouvée avec ce nom.").to_string())
            } else {
                Err(by_lang(lang,
                    "Several people match that name — please use the full given and family name.",
                    "Plusieurs personnes correspondent à ce nom — veuillez utiliser le prénom et le nom complets.").to_string())
            }
        },
        Err(e) => Err(e.to_string()),
    }
}

fn person_from_form(form: &PersonForm, id: Option<&str>) -> serde_json::Value {
    json!({
        "id": id,
        "userEmail": form.user_email,
        "familyName": form.family_name,
        "givenName": form.given_name,
        "email": form.email,
        "phone": form.phone,
        "workAddress": form.work_address,
        "city": form.city,
        "province": form.province,
        "postalCode": form.postal_code,
        "country": form.country,
        "peoplesoftId": form.peoplesoft_id,
        "orcidId": form.orcid_id,
        "personnelType": form.personnel_type,
        "organization": {"id": form.organization_id},
    })
}

#[get("/{lang}/person_by_name/{name}")]
pub async fn person_by_name(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req:HttpRequest) -> impl Responder {
    let (lang, name) = path_params.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = session_bearer(&req.get_session());

    let people = match get_people_by_name(name, bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/people", &lang));
        },
    };

    ctx.insert("people", &people.person_by_name);

    render_page(&data, "person/person_by_name.html", &ctx)
}


#[get("/{lang}/person/{person_id}")]
pub async fn person_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req:HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = session_bearer(&req.get_session());

    let r = match get_person_by_id(person_id.clone(), bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/people", &lang));
        },
    };

    // Capability radar: best level per domain (validated as the filled series,
    // self-identified as a lighter dashed series). Rendered only when the
    // person has at least 3 distinct domains, since a radar needs ≥3 axes.
    let person = &r.person_by_id;
    let mut dom_validated: BTreeMap<String, i64> = BTreeMap::new();
    let mut dom_self: BTreeMap<String, i64> = BTreeMap::new();
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
        let e = dom_self.entry(domain.clone()).or_insert(0);
        if self_w > *e { *e = self_w; }
        let e = dom_validated.entry(domain).or_insert(0);
        if val_w > *e { *e = val_w; }
    }

    let domains: Vec<String> = dom_self.keys().cloned().collect();
    if domains.len() >= 3 {
        let indicators: Vec<serde_json::Value> = domains.iter()
            .map(|d| json!({"name": domain_short_label(d), "max": 5}))
            .collect();
        let val_series: Vec<i64> = domains.iter().map(|d| *dom_validated.get(d).unwrap_or(&0)).collect();
        let self_series: Vec<i64> = domains.iter().map(|d| *dom_self.get(d).unwrap_or(&0)).collect();
        let radar = json!({
            "tooltip": {},
            "legend": {"bottom": 0, "data": ["Validated", "Self-identified"]},
            "radar": {"indicator": indicators, "radius": "65%"},
            "series": [{
                "type": "radar",
                "data": [
                    {"value": val_series, "name": "Validated", "areaStyle": {"opacity": 0.2}},
                    {"value": self_series, "name": "Self-identified", "lineStyle": {"type": "dashed"}}
                ]
            }]
        });
        ctx.insert("capability_radar", &chart_json(&radar));
    }

    ctx.insert("person", &r.person_by_id);

    // Overdue work ids for the badge: due before today and not finished.
    // (Tera can't compare date strings, so membership is computed here.)
    let today = chrono::Utc::now().date_naive();
    let overdue_work_ids: Vec<&String> = r.person_by_id.active_roles.iter()
        .flat_map(|role| role.work.iter())
        .filter(|w| {
            let status = serde_json::to_value(&w.work_status).ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_default();
            status != "COMPLETED" && status != "CANCELLED"
        })
        .filter(|w| w.due_date.map(|d| d.date() < today).unwrap_or(false))
        .map(|w| &w.id)
        .collect();
    ctx.insert("overdue_work_ids", &overdue_work_ids);

    // Self-service band: is this record the signed-in user's own person?
    // Resolved through `me` (ownership-based) so it works for plain users,
    // not just admins.
    let is_self = if bearer.is_empty() {
        false
    } else {
        get_me(bearer.clone(), &data.api_url, Arc::clone(&data.client)).await
            .ok()
            .and_then(|r| r.me.person)
            .map(|p| p.id == person_id)
            .unwrap_or(false)
    };
    ctx.insert("is_self", &is_self);

    // Account status drives the status chip and the Grant-access action. The
    // userByEmail lookup is admin-guarded, so this only resolves for admins;
    // it degrades silently otherwise.
    if let Ok(u) = get_user_by_email(person.email.clone(), bearer, &data.api_url, Arc::clone(&data.client)).await {
        ctx.insert("account_status", &u.user_by_email.status);
        ctx.insert("account_user_id", &u.user_by_email.id);
    }

    render_page(&data, "person/person.html", &ctx)
}

#[derive(Deserialize, Debug)]
pub struct GrantAccessForm {
    pub csrf_token: String,
}

/// Issue an activation invite for a person's account (operator+). Uses the
/// invitePerson mutation, which resolves the account server-side, so operators
/// who cannot read user records can still grant access. Surfaces the activation
/// link as a flash message to share.
#[post("/{lang}/person/{person_id}/grant-access")]
pub async fn grant_access_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<GrantAccessForm>,
    req: HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/person/{}", &lang, &person_id));
    }

    match crate::graphql::invite_person(person_id.clone(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(resp) => {
            let link = format!("/{}/activate?token={}", &lang, resp.invite_person.activation_token);
            security::add_flash(
                &session,
                "success",
                &by_lang(
                    &lang,
                    &format!("Access granted. Share this activation link with the person: {}", link),
                    &format!("Accès accordé. Partagez ce lien d'activation avec la personne : {}", link),
                ),
            );
        },
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/person/{}", &lang, &person_id))
}

#[get("/{lang}/person/new")]
pub async fn create_person_form(
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
    ctx.insert("person", &json!({
        "userEmail": "", "familyName": "", "givenName": "", "email": "", "phone": "",
        "workAddress": "", "city": "", "province": "", "postalCode": "", "country": "",
        "peoplesoftId": "", "orcidId": "", "personnelType": "",
        "organization": {"id": ""},
    }));
    ctx.insert("organization_options", &organization_options(&auth.bearer, &data).await);
    ctx.insert("personnel_types", &personnel_type_options());

    render_page(&data, "person/person_form.html", &ctx)
}

#[post("/{lang}/person/new")]
pub async fn create_person_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    form: web::Form<PersonForm>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/person/new", &lang));
    }

    let render_error = |ctx_error: String, id: Option<Identity>, organization_options: serde_json::Value| {
        // Flash must be queued before generate_basic_context drains the queue
        security::add_flash(&session, "danger", &ctx_error);
        let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
        ctx.insert("edit", &false);
        ctx.insert("person", &person_from_form(&form, None));
        ctx.insert("organization_options", &organization_options);
        ctx.insert("personnel_types", &personnel_type_options());
        render_page(&data, "person/person_form.html", &ctx)
    };

    // Email is required: creating a Person now auto-provisions a (login-disabled)
    // user account, and the email becomes that account's login id.
    if form.email.trim().is_empty() {
        let message = by_lang(
            &lang,
            "An email address is required — it becomes the person's account login.",
            "Une adresse courriel est requise — elle devient l'identifiant du compte de la personne.",
        ).to_string();
        let options = organization_options(&auth.bearer, &data).await;
        return render_error(message, id, options);
    }

    let new_person = create_person::NewPersonInput {
        family_name: form.family_name.trim().to_string(),
        given_name: form.given_name.trim().to_string(),
        email: form.email.trim().to_string(),
        phone: form.phone.trim().to_string(),
        work_address: form.work_address.trim().to_string(),
        city: form.city.trim().to_string(),
        province: form.province.trim().to_string(),
        postal_code: form.postal_code.trim().to_string(),
        country: form.country.trim().to_string(),
        organization_id: form.organization_id.clone(),
        peoplesoft_id: form.peoplesoft_id.trim().to_string(),
        orcid_id: form.orcid_id.trim().to_string(),
        personnel_type: serde_json::from_value(json!(form.personnel_type))
            .expect("PersonnelType deserialization is infallible"),
    };

    match create_person(new_person, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Person created.", "Personne créée."),
            );
            redirect_to(format!("/{}/person/{}", &lang, response.create_person.id))
        },
        Err(e) => {
            let options = organization_options(&auth.bearer, &data).await;
            render_error(e.to_string(), id, options)
        },
    }
}

#[get("/{lang}/person/{person_id}/edit")]
pub async fn edit_person_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_person_by_id(person_id, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("edit", &true);
    ctx.insert("person", &r.person_by_id);
    ctx.insert("organization_options", &organization_options(&auth.bearer, &data).await);
    ctx.insert("personnel_types", &personnel_type_options());

    render_page(&data, "person/person_form.html", &ctx)
}

#[post("/{lang}/person/{person_id}/edit")]
pub async fn edit_person_post(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<PersonForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/person/{}/edit", &lang, &person_id));
    }

    let person_data = update_person::PersonData {
        id: person_id.clone(),
        user_id: None,
        family_name: Some(form.family_name.trim().to_string()),
        given_name: Some(form.given_name.trim().to_string()),
        email: Some(form.email.trim().to_string()),
        phone: Some(form.phone.trim().to_string()),
        work_address: Some(form.work_address.trim().to_string()),
        city: Some(form.city.trim().to_string()),
        province: Some(form.province.trim().to_string()),
        postal_code: Some(form.postal_code.trim().to_string()),
        country: Some(form.country.trim().to_string()),
        organization_id: Some(form.organization_id.clone()),
        peoplesoft_id: Some(form.peoplesoft_id.trim().to_string()),
        orcid_id: Some(form.orcid_id.trim().to_string()),
        personnel_type: Some(serde_json::from_value(json!(form.personnel_type))
            .expect("PersonnelType deserialization is infallible")),
        updated_at: None,
        retired_at: None,
    };

    match update_person(person_data, auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Person updated.", "Personne mise à jour."),
            );
            redirect_to(format!("/{}/person/{}", &lang, response.update_person.id))
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());

            let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
            ctx.insert("edit", &true);
            ctx.insert("person", &person_from_form(&form, Some(&person_id)));
            ctx.insert("organization_options", &organization_options(&auth.bearer, &data).await);
            ctx.insert("personnel_types", &personnel_type_options());

            render_page(&data, "person/person_form.html", &ctx)
        },
    }
}

#[get("/{lang}/person/{person_id}/retire")]
pub async fn retire_person_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let r = match get_person_by_id(person_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("person", &r.person_by_id);

    render_page(&data, "person/person_retire.html", &ctx)
}

#[post("/{lang}/person/{person_id}/retire")]
pub async fn retire_person_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<RetireForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/person/{}", &lang, &person_id));
    }

    let person_data = update_person::PersonData {
        id: person_id.clone(),
        user_id: None,
        family_name: None,
        given_name: None,
        email: None,
        phone: None,
        work_address: None,
        city: None,
        province: None,
        postal_code: None,
        country: None,
        organization_id: None,
        peoplesoft_id: None,
        orcid_id: None,
        personnel_type: None,
        updated_at: None,
        retired_at: Some(chrono::Utc::now().naive_utc()),
    };

    match update_person(person_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => {
            security::add_flash(
                &session,
                "success",
                by_lang(&lang, "Person retired.", "Personne retirée."),
            );
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
        },
    };

    redirect_to(format!("/{}/person/{}", &lang, &person_id))
}

#[derive(Deserialize, Debug)]
pub struct AffiliationForm {
    pub csrf_token: String,
    pub organization_id: String,
    pub affiliation_role: String,
    #[serde(default)]
    pub end_date: String,
}

#[derive(Deserialize, Debug)]
pub struct EndAffiliationForm {
    pub csrf_token: String,
}

fn parse_date(value: &str) -> Option<chrono::NaiveDateTime> {
    chrono::NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
}

#[get("/{lang}/person/{person_id}/affiliation/new")]
pub async fn create_affiliation_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let person = match get_person_by_id(person_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.person_by_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("person", &person);
    ctx.insert("affiliation", &json!({"organization": {"id": ""}, "affiliationRole": "", "endDate": ""}));
    ctx.insert("organization_options", &organization_options(&auth.bearer, &data).await);

    render_page(&data, "person/affiliation_form.html", &ctx)
}

#[post("/{lang}/person/{person_id}/affiliation/new")]
pub async fn create_affiliation_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<AffiliationForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/person/{}/affiliation/new", &lang, &person_id));
    }

    // homeOrgId is the person's own organization
    let home_org_id = match get_person_by_id(person_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.person_by_id.organization.id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/person/{}", &lang, &person_id));
        },
    };

    let new_affiliation = create_affiliation::NewAffiliation {
        person_id: person_id.clone(),
        organization_id: form.organization_id.clone(),
        home_org_id,
        affiliation_role: form.affiliation_role.trim().to_string(),
        end_date: parse_date(&form.end_date),
    };

    match create_affiliation(new_affiliation, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Affiliation added.", "Affiliation ajoutée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/person/{}", &lang, &person_id))
}

#[post("/{lang}/person/{person_id}/affiliation/{affiliation_id}/end")]
pub async fn end_affiliation_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String, String)>,
    form: web::Form<EndAffiliationForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id, affiliation_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/person/{}", &lang, &person_id));
    }

    let affiliation_data = update_affiliation::AffiliationData {
        id: affiliation_id,
        affiliation_role: None,
        start_datestamp: None,
        end_date: Some(chrono::Utc::now().naive_utc()),
    };

    match update_affiliation(affiliation_data, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Affiliation ended.", "Affiliation terminée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/person/{}", &lang, &person_id))
}

/// LanguageName enum values, kept in sync with the API schema.
pub const LANGUAGE_NAMES: [&str; 10] = [
    "ENGLISH", "FRENCH", "ARABIC", "CHINESE", "SPANISH",
    "GERMAN", "JAPANESE", "KOREAN", "ITALIAN", "OTHER",
];

/// LanguageLevel enum values (Canadian government scale; X = none).
pub const LANGUAGE_LEVELS: [&str; 5] = ["A", "B", "C", "E", "X"];

#[derive(Deserialize, Debug)]
pub struct LanguageForm {
    pub csrf_token: String,
    pub language_name: String,
    #[serde(default)]
    pub reading: String,
    #[serde(default)]
    pub writing: String,
    #[serde(default)]
    pub speaking: String,
}

fn language_name_options() -> serde_json::Value {
    json!(LANGUAGE_NAMES.iter()
        .map(|n| {
            let lower = n.to_lowercase();
            let mut chars = lower.chars();
            let label = match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            };
            json!({"value": n, "label": label})
        })
        .collect::<Vec<serde_json::Value>>())
}

fn language_level_options() -> serde_json::Value {
    json!(LANGUAGE_LEVELS.iter().map(|l| json!({"value": l, "label": l})).collect::<Vec<serde_json::Value>>())
}

/// Parse a blank-or-level string into the generated LanguageLevel enum.
/// Blank means "not specified" (None).
fn parse_level(value: &str) -> Option<create_language_data::LanguageLevel> {
    if value.trim().is_empty() {
        None
    } else {
        serde_json::from_value(json!(value)).ok()
    }
}

#[get("/{lang}/person/{person_id}/language/new")]
pub async fn create_language_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let person = match get_person_by_id(person_id.clone(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.person_by_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("person", &person);
    ctx.insert("language_names", &language_name_options());
    ctx.insert("language_levels", &language_level_options());

    render_page(&data, "person/language_form.html", &ctx)
}

#[post("/{lang}/person/{person_id}/language/new")]
pub async fn create_language_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<LanguageForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/person/{}/language/new", &lang, &person_id));
    }

    let new_language = create_language_data::NewLanguageData {
        person_id: person_id.clone(),
        language_name: serde_json::from_value(json!(form.language_name)).expect("LanguageName deserialization is infallible"),
        reading: parse_level(&form.reading),
        writing: parse_level(&form.writing),
        speaking: parse_level(&form.speaking),
    };

    match create_language_data(new_language, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Language added.", "Langue ajoutée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/person/{}", &lang, &person_id))
}


#[derive(Deserialize, Debug)]
pub struct PeopleIndexParams {
    #[serde(default)]
    pub retired: String,
    #[serde(default)]
    pub q: String,
    /// Organization UUID to filter by; empty means all organizations.
    #[serde(default)]
    pub org: String,
    /// "in_role" | "available" | "" (all). Whether the person currently holds
    /// an active role.
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub page: String,
}

/// People shown per page. The API now filters and paginates server-side.
const PEOPLE_PAGE_SIZE: i64 = 100;

/// How many rows an index renders before truncating with a "refine search"
/// hint. Used across the People/Teams/Roles indexes.
pub const INDEX_PAGE_CAP: usize = 100;


#[get("/{lang}/people")]
pub async fn person_index(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    params: web::Query<PeopleIndexParams>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = session_bearer(&req.get_session());

    let show_retired = params.retired == "1";
    let selected_org = params.org.trim().to_string();
    let selected_status = params.status.trim().to_string();
    let search = { let q = params.q.trim(); if q.is_empty() { None } else { Some(q.to_string()) } };
    let org_filter = if selected_org.is_empty() { None } else { Some(selected_org.clone()) };
    let status_filter = if selected_status.is_empty() { None } else { Some(selected_status.clone()) };
    let page = params.page.trim().parse::<i64>().unwrap_or(1).max(1);
    let offset = (page - 1) * PEOPLE_PAGE_SIZE;

    // The API filters (search + org + role status + retired) and paginates
    // server-side; the org dropdown is independent, so fetch it concurrently.
    let (people_res, orgs_res) = futures::join!(
        all_people(search, org_filter, status_filter, show_retired, Some(PEOPLE_PAGE_SIZE), offset, bearer.clone(), &data.api_url, Arc::clone(&data.client)),
        all_organizations(bearer, &data.api_url, Arc::clone(&data.client)),
    );
    let (people, total) = match people_res {
        Ok(r) => (r.all_people, r.people_count),
        Err(_) => (Vec::new(), 0),
    };

    // Organization filter options, active orgs only, sorted by name.
    let mut organizations = orgs_res.map(|r| r.all_organizations).unwrap_or_default();
    organizations.retain(|o| o.retired_at.is_none());
    organizations.sort_by(|a, b| a.name_en.to_lowercase().cmp(&b.name_en.to_lowercase()));

    let total_pages = ((total + PEOPLE_PAGE_SIZE - 1) / PEOPLE_PAGE_SIZE).max(1);

    ctx.insert("people", &people);
    ctx.insert("total", &total);
    ctx.insert("page", &page);
    ctx.insert("total_pages", &total_pages);
    ctx.insert("has_prev", &(page > 1));
    ctx.insert("has_next", &(page < total_pages));
    ctx.insert("q", &params.q);
    ctx.insert("show_retired", &show_retired);
    ctx.insert("organizations", &organizations);
    ctx.insert("selected_org", &selected_org);
    ctx.insert("selected_status", &selected_status);

    // HTMX search requests get just the list partial to swap in place
    let template = if is_htmx(&req) { "person/person_list.html" } else { "person/person_index.html" };
    render_page(&data, template, &ctx)
}

#[post("/{lang}/person/{person_id}/restore")]
pub async fn restore_person_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<RetireForm>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/person/{}", &lang, &person_id));
    }

    match restore_person(person_id.clone(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success", by_lang(&lang, "Person restored.", "Personne restaurée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/person/{}", &lang, &person_id))
}

/// Confirmation step before ending an affiliation.
#[get("/{lang}/person/{person_id}/affiliation/{affiliation_id}/end")]
pub async fn end_affiliation_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String, String)>,

    req: HttpRequest) -> impl Responder {
    let (lang, person_id, affiliation_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let person = match get_person_by_id(person_id.clone(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.person_by_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/people", &lang));
        },
    };

    let person_name = format!("{} {}", person.given_name, person.family_name);
    let org_name = person.affiliations.iter()
        .find(|a| a.id == affiliation_id)
        .map(|a| a.organization.name_en.clone())
        .unwrap_or_else(|| by_lang(&lang, "the affiliated organization", "l'organisation affiliée").to_string());

    let message = if lang == "fr" {
        format!("L'affiliation de {} avec {} prendra fin.", person_name, org_name)
    } else {
        format!("{}'s affiliation with {} will be ended.", person_name, org_name)
    };

    let ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    render_confirm(
        &data, &req, ctx,
        by_lang(&lang, "End affiliation", "Mettre fin à l'affiliation"),
        &message,
        None,
        &format!("/{}/person/{}/affiliation/{}/end", &lang, &person_id, &affiliation_id),
        by_lang(&lang, "End affiliation", "Mettre fin à l'affiliation"),
        &format!("/{}/person/{}", &lang, &person_id),
    )
}

#[derive(Deserialize, Debug)]
pub struct PersonOptionsParams {
    /// The typeahead sends the bound field's own name via hx-include, so
    /// accept the common field names as the query.
    #[serde(default)]
    pub q: String,
    #[serde(default)]
    pub person_name: String,
}

/// HTMX datalist options for person-name typeahead fields (see the
/// forms::person_picker macro). Returns bare <option> elements; without JS
/// the bound input is still a plain text field the server resolves by name.
#[get("/{lang}/person_options")]
pub async fn person_options_datalist(
    data: web::Data<AppData>,
    path: web::Path<String>,
    params: web::Query<PersonOptionsParams>,

    req: HttpRequest) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::User) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let raw = if !params.q.trim().is_empty() { params.q.trim() } else { params.person_name.trim() };
    if raw.chars().count() < 2 {
        return HttpResponse::Ok().body("");
    }

    // Search on the most discriminating token so a partially typed
    // "given fam" still narrows on the family name.
    let token = raw.split_whitespace().last().unwrap_or(raw).to_string();

    let escape = |s: &str| s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;");

    match get_people_by_name(token, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => {
            let options: String = r.person_by_name.iter()
                .take(10)
                .map(|p| format!("<option value=\"{} {}\"></option>", escape(&p.given_name), escape(&p.family_name)))
                .collect();
            HttpResponse::Ok().body(options)
        },
        Err(_) => HttpResponse::Ok().body(""),
    }
}
