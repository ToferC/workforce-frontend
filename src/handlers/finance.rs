use std::sync::Arc;

use actix_web::{web, get, post, HttpRequest, Responder};
use actix_identity::Identity;
use actix_session::SessionExt;
use serde::Deserialize;
use serde_json::json;

use crate::{generate_basic_context, by_lang, AppData};
use crate::graphql::{
    create_contract, update_contract, delete_contract, get_contract_by_id,
    all_pay_rates, create_pay_rate, set_budget_allocation,
};
use crate::handlers::utility::{redirect_to, csrf_failure_flash, session_bearer, render_page, render_confirm};
use crate::security::{self, MinimumRole};
use super::role::{RANKS, OCCUPATIONAL_GROUPS};
use super::admin::CsrfOnlyForm;

/// ContractStatus enum values, kept in sync with the API schema.
pub const CONTRACT_STATUSES: [&str; 3] = ["PLANNED", "ACTIVE", "CLOSED"];

fn humanize(value: &str) -> String {
    let mut out = String::new();
    for (i, part) in value.split('_').enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.push(first);
            out.push_str(&chars.as_str().to_lowercase());
        }
    }
    out
}

fn enum_options(values: &[&str]) -> serde_json::Value {
    json!(values
        .iter()
        .map(|value| json!({"value": value, "label": humanize(value)}))
        .collect::<Vec<serde_json::Value>>())
}

/// Dollars typed into a form -> integer cents, honouring the page language's
/// separators: English "1,234,567.89" (comma groups, period decimal), French
/// "1 234 567,89" (space or period groups, comma decimal). Anything that
/// doesn't parse as a number after normalization is rejected.
fn dollars_to_cents(input: &str, lang: &str) -> Option<i64> {
    let mut cleaned: String = input
        .trim()
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '$' && *c != '\u{a0}' && *c != '\u{202f}')
        .collect();
    if lang == "fr" {
        cleaned = cleaned.replace('.', "").replace(',', ".");
    } else {
        cleaned = cleaned.replace(',', "");
    }
    let value: f64 = cleaned.parse().ok()?;
    if !value.is_finite() {
        return None;
    }
    Some((value * 100.0).round() as i64)
}

#[cfg(test)]
mod tests {
    use super::dollars_to_cents;

    #[test]
    fn parses_english_separators() {
        assert_eq!(dollars_to_cents("1,234,567.89", "en"), Some(123_456_789));
        assert_eq!(dollars_to_cents("1000", "en"), Some(100_000));
        assert_eq!(dollars_to_cents("$250,000", "en"), Some(25_000_000));
        assert_eq!(dollars_to_cents("abc", "en"), None);
        assert_eq!(dollars_to_cents("1.2.3", "en"), None);
    }

    #[test]
    fn parses_french_separators() {
        // Comma is the decimal separator in French — the bug this guards
        // against read "1000,50" as $100,050.
        assert_eq!(dollars_to_cents("1000,50", "fr"), Some(100_050));
        assert_eq!(dollars_to_cents("1\u{a0}234\u{a0}567,89", "fr"), Some(123_456_789));
        assert_eq!(dollars_to_cents("1.234.567,89", "fr"), Some(123_456_789));
        assert_eq!(dollars_to_cents("1000", "fr"), Some(100_000));
    }
}

/// "YYYY-MM-DD" from a date input -> NaiveDateTime at midnight.
fn parse_form_date(input: &str) -> Option<chrono::NaiveDateTime> {
    chrono::NaiveDate::parse_from_str(input.trim(), "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
}

// ---------------------------------------------------------------------------
// Contracts (under a task)
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
pub struct ContractForm {
    pub csrf_token: String,
    pub reference_number: String,
    pub vendor: String,
    pub description: String,
    pub start_date: String,
    pub end_date: String,
    pub total_value: String,
    pub status: String,
}

#[get("/{lang}/task/{task_id}/contract/new")]
pub async fn create_contract_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    req: HttpRequest,
) -> impl Responder {
    let (lang, task_id) = path_params.into_inner();
    let session = req.get_session();

    if let Err(response) = security::require_role(&session, &lang, MinimumRole::Operator) {
        return response;
    }

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("task_id", &task_id);
    ctx.insert("contract_statuses", &enum_options(&CONTRACT_STATUSES));
    ctx.insert("today", &chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string());

    render_page(&data, "finance/contract_form.html", &ctx)
}

#[post("/{lang}/task/{task_id}/contract/new")]
pub async fn create_contract_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<ContractForm>,
    req: HttpRequest,
) -> impl Responder {
    let (lang, task_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/task/{}/contract/new", &lang, &task_id));
    }

    let (start, end, cents) = (
        parse_form_date(&form.start_date),
        parse_form_date(&form.end_date),
        dollars_to_cents(&form.total_value, &lang),
    );
    let (Some(start), Some(end), Some(cents)) = (start, end, cents) else {
        security::add_flash(&session, "danger", by_lang(&lang,
            "Enter valid start and end dates and a contract value.",
            "Entrez des dates de début et de fin valides ainsi qu'une valeur de contrat."));
        return redirect_to(format!("/{}/task/{}/contract/new", &lang, &task_id));
    };
    if end < start {
        security::add_flash(&session, "danger", by_lang(&lang,
            "The end date cannot precede the start date.",
            "La date de fin ne peut pas précéder la date de début."));
        return redirect_to(format!("/{}/task/{}/contract/new", &lang, &task_id));
    }

    let new_contract = create_contract::NewContract {
        task_id: task_id.clone(),
        reference_number: form.reference_number.trim().to_string(),
        vendor: form.vendor.trim().to_string(),
        description: form.description.trim().to_string(),
        start_date: start,
        end_date: end,
        total_value_cents: cents,
        status: serde_json::from_value(json!(form.status))
            .unwrap_or(create_contract::ContractStatus::ACTIVE),
    };

    match create_contract(new_contract, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success",
            by_lang(&lang, "Contract recorded.", "Contrat enregistré.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/task/{}", &lang, &task_id))
}

#[get("/{lang}/contract/{contract_id}/edit")]
pub async fn edit_contract_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    req: HttpRequest,
) -> impl Responder {
    let (lang, contract_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let contract = match get_contract_by_id(contract_id.clone(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.contract_by_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/tasks", &lang));
        },
    };

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("task_id", &contract.task_id);
    ctx.insert("contract", &json!({
        "id": contract.id,
        "referenceNumber": contract.reference_number,
        "vendor": contract.vendor,
        "description": contract.description,
        "startDate": contract.start_date.format("%Y-%m-%d").to_string(),
        "endDate": contract.end_date.format("%Y-%m-%d").to_string(),
        "totalValueDollars": format!("{:.2}", contract.total_value_cents as f64 / 100.0),
        "status": format!("{:?}", contract.status),
    }));
    ctx.insert("contract_statuses", &enum_options(&CONTRACT_STATUSES));

    render_page(&data, "finance/contract_form.html", &ctx)
}

#[post("/{lang}/contract/{contract_id}/edit")]
pub async fn edit_contract_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<ContractForm>,
    req: HttpRequest,
) -> impl Responder {
    let (lang, contract_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/contract/{}/edit", &lang, &contract_id));
    }

    let (start, end, cents) = (
        parse_form_date(&form.start_date),
        parse_form_date(&form.end_date),
        dollars_to_cents(&form.total_value, &lang),
    );
    let (Some(start), Some(end), Some(cents)) = (start, end, cents) else {
        security::add_flash(&session, "danger", by_lang(&lang,
            "Enter valid start and end dates and a contract value.",
            "Entrez des dates de début et de fin valides ainsi qu'une valeur de contrat."));
        return redirect_to(format!("/{}/contract/{}/edit", &lang, &contract_id));
    };
    if end < start {
        security::add_flash(&session, "danger", by_lang(&lang,
            "The end date cannot precede the start date.",
            "La date de fin ne peut pas précéder la date de début."));
        return redirect_to(format!("/{}/contract/{}/edit", &lang, &contract_id));
    }

    let update = update_contract::ContractUpdate {
        id: contract_id.clone(),
        reference_number: Some(form.reference_number.trim().to_string()),
        vendor: Some(form.vendor.trim().to_string()),
        description: Some(form.description.trim().to_string()),
        start_date: Some(start),
        end_date: Some(end),
        total_value_cents: Some(cents),
        status: serde_json::from_value(json!(form.status)).ok(),
    };

    let task_id = match update_contract(update, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => {
            security::add_flash(&session, "success",
                by_lang(&lang, "Contract updated.", "Contrat mis à jour."));
            r.update_contract.task_id
        },
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/contract/{}/edit", &lang, &contract_id));
        },
    };

    redirect_to(format!("/{}/task/{}", &lang, &task_id))
}

#[get("/{lang}/contract/{contract_id}/delete")]
pub async fn delete_contract_form(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    req: HttpRequest,
) -> impl Responder {
    let (lang, contract_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    let contract = match get_contract_by_id(contract_id.clone(), auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.contract_by_id,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}/tasks", &lang));
        },
    };

    let ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    let message = by_lang(&lang,
        "This removes the contract and its expenditures from all projections.",
        "Cette action retire le contrat et ses dépenses de toutes les projections.");
    render_confirm(
        &data,
        &req,
        ctx,
        by_lang(&lang, "Delete contract?", "Supprimer le contrat?"),
        &format!("{} — {}", contract.reference_number, contract.vendor),
        Some(message),
        &format!("/{}/contract/{}/delete", &lang, &contract_id),
        by_lang(&lang, "Delete contract", "Supprimer le contrat"),
        &format!("/{}/task/{}", &lang, &contract.task_id),
    )
}

#[post("/{lang}/contract/{contract_id}/delete")]
pub async fn delete_contract_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<CsrfOnlyForm>,
    req: HttpRequest,
) -> impl Responder {
    let (lang, contract_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/tasks", &lang));
    }

    // Fetch first so we know where to send the user afterwards.
    let task_id = get_contract_by_id(contract_id.clone(), auth.bearer.clone(), &data.api_url, Arc::clone(&data.client))
        .await
        .map(|r| r.contract_by_id.task_id)
        .ok();

    match delete_contract(contract_id, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success",
            by_lang(&lang, "Contract deleted.", "Contrat supprimé.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    match task_id {
        Some(task_id) => redirect_to(format!("/{}/task/{}", &lang, &task_id)),
        None => redirect_to(format!("/{}/tasks", &lang)),
    }
}

// ---------------------------------------------------------------------------
// Pay rates (admin)
// ---------------------------------------------------------------------------

/// Civilian and military rates are entered through separate forms, so each
/// form only carries its own classification fields. Every gcds-* field gets
/// `#[serde(default)]`: an untouched form-associated GCDS component submits
/// no value at all (not an empty string), and without the default actix
/// rejects the whole POST with an opaque "missing field" parse error.
#[derive(Deserialize, Debug)]
pub struct CivilianRateForm {
    pub csrf_token: String,
    #[serde(default)]
    pub occupational_group: String,
    #[serde(default)]
    pub occupational_level: String,
    #[serde(default)]
    pub annual_rate: String,
    #[serde(default)]
    pub effective_date: String,
}

#[derive(Deserialize, Debug)]
pub struct MilitaryRateForm {
    pub csrf_token: String,
    #[serde(default)]
    pub rank: String,
    #[serde(default)]
    pub annual_rate: String,
    #[serde(default)]
    pub effective_date: String,
}

#[get("/{lang}/admin/pay_rates")]
pub async fn pay_rates_admin(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let lang = path_params.into_inner();
    let session = req.get_session();

    if let Err(response) = security::require_role(&session, &lang, MinimumRole::Admin) {
        return response;
    }

    let bearer = session_bearer(&session);
    let rates = match all_pay_rates(bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => r.pay_rates,
        Err(e) => {
            security::add_flash(&session, "danger", &e.to_string());
            return redirect_to(format!("/{}", &lang));
        },
    };

    // Split civilian and military rates for the two tables. Civilian rates
    // read best grouped by classification (newest rate first within one);
    // military rates in order of seniority, which is RANKS's array order.
    let mut civilian: Vec<serde_json::Value> = Vec::new();
    let mut military: Vec<serde_json::Value> = Vec::new();
    let mut rates = rates;
    rates.sort_by(|a, b| {
        match (&a.rank, &b.rank) {
            (Some(ra), Some(rb)) => {
                let seniority = |r: &str| RANKS.iter().position(|v| *v == r).unwrap_or(usize::MAX);
                seniority(&format!("{:?}", ra))
                    .cmp(&seniority(&format!("{:?}", rb)))
                    .then(b.effective_date.cmp(&a.effective_date))
            }
            _ => a
                .occupational_group
                .as_ref()
                .map(|g| format!("{:?}", g))
                .cmp(&b.occupational_group.as_ref().map(|g| format!("{:?}", g)))
                .then(a.occupational_level.cmp(&b.occupational_level))
                .then(b.effective_date.cmp(&a.effective_date)),
        }
    });
    for rate in &rates {
        let row = json!({
            "id": rate.id,
            "group": rate.occupational_group.as_ref().map(|g| humanize(&format!("{:?}", g))),
            "level": rate.occupational_level,
            "rank": rate.rank.as_ref().map(|r| humanize(&format!("{:?}", r))),
            "annualRateCents": rate.annual_rate_cents,
            "effectiveDate": rate.effective_date.format("%Y-%m-%d").to_string(),
        });
        if rate.rank.is_some() {
            military.push(row);
        } else {
            civilian.push(row);
        }
    }

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);
    ctx.insert("civilian_rates", &civilian);
    ctx.insert("military_rates", &military);
    ctx.insert("occupational_groups", &enum_options(&OCCUPATIONAL_GROUPS));
    ctx.insert("ranks", &enum_options(&RANKS));
    ctx.insert("today", &chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string());

    render_page(&data, "finance/pay_rates.html", &ctx)
}

/// Rate and effective date validation shared by both pay-rate forms.
/// Returns a flash message (already localized) on failure.
fn parse_rate_and_date(
    annual_rate: &str,
    effective_date: &str,
    lang: &str,
) -> Result<(i64, chrono::NaiveDateTime), String> {
    match (dollars_to_cents(annual_rate, lang), parse_form_date(effective_date)) {
        (Some(cents), _) if cents <= 0 => Err(by_lang(lang,
            "The annual rate must be greater than zero.",
            "Le taux annuel doit être supérieur à zéro.").to_string()),
        (Some(cents), Some(effective)) => Ok((cents, effective)),
        (None, _) => Err(by_lang(lang,
            "Enter the annual rate in dollars, e.g. 95,000.",
            "Entrez le taux annuel en dollars, p. ex. 95 000.").to_string()),
        (_, None) => Err(by_lang(lang,
            "Enter a valid effective date.",
            "Entrez une date d'entrée en vigueur valide.").to_string()),
    }
}

async fn submit_pay_rate(
    data: &web::Data<AppData>,
    session: &actix_session::Session,
    lang: &str,
    bearer: String,
    new_rate: create_pay_rate::NewPayRate,
) -> actix_web::HttpResponse {
    match create_pay_rate(new_rate, bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(session, "success",
            by_lang(lang, "Pay rate added.", "Taux de rémunération ajouté.")),
        Err(e) => security::add_flash(session, "danger", &e.to_string()),
    };
    redirect_to(format!("/{}/admin/pay_rates", lang))
}

#[post("/{lang}/admin/pay_rates/civilian")]
pub async fn pay_rate_civilian_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<String>,
    form: web::Form<CivilianRateForm>,
    req: HttpRequest,
) -> impl Responder {
    let lang = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Admin) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/admin/pay_rates", &lang));
    }

    let group: Option<create_pay_rate::OccupationalGroup> =
        serde_json::from_value(json!(form.occupational_group.trim())).ok();
    let Some(group) = group else {
        security::add_flash(&session, "danger", by_lang(&lang,
            "Choose an occupational group.",
            "Choisissez un groupe professionnel."));
        return redirect_to(format!("/{}/admin/pay_rates", &lang));
    };
    let level = form.occupational_level.trim().parse::<i64>().ok().filter(|l| (1..=15).contains(l));
    let Some(level) = level else {
        security::add_flash(&session, "danger", by_lang(&lang,
            "Enter a level between 1 and 15.",
            "Entrez un niveau entre 1 et 15."));
        return redirect_to(format!("/{}/admin/pay_rates", &lang));
    };
    let (cents, effective) = match parse_rate_and_date(&form.annual_rate, &form.effective_date, &lang) {
        Ok(parsed) => parsed,
        Err(message) => {
            security::add_flash(&session, "danger", &message);
            return redirect_to(format!("/{}/admin/pay_rates", &lang));
        }
    };

    let new_rate = create_pay_rate::NewPayRate {
        occupational_group: Some(group),
        occupational_level: Some(level),
        rank: None,
        annual_rate_cents: cents,
        effective_date: effective,
    };
    submit_pay_rate(&data, &session, &lang, auth.bearer, new_rate).await
}

#[post("/{lang}/admin/pay_rates/military")]
pub async fn pay_rate_military_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<String>,
    form: web::Form<MilitaryRateForm>,
    req: HttpRequest,
) -> impl Responder {
    let lang = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Admin) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/admin/pay_rates", &lang));
    }

    let rank: Option<create_pay_rate::Rank> =
        serde_json::from_value(json!(form.rank.trim())).ok();
    let Some(rank) = rank else {
        security::add_flash(&session, "danger", by_lang(&lang,
            "Choose a rank.",
            "Choisissez un grade."));
        return redirect_to(format!("/{}/admin/pay_rates", &lang));
    };
    let (cents, effective) = match parse_rate_and_date(&form.annual_rate, &form.effective_date, &lang) {
        Ok(parsed) => parsed,
        Err(message) => {
            security::add_flash(&session, "danger", &message);
            return redirect_to(format!("/{}/admin/pay_rates", &lang));
        }
    };

    let new_rate = create_pay_rate::NewPayRate {
        occupational_group: None,
        occupational_level: None,
        rank: Some(rank),
        annual_rate_cents: cents,
        effective_date: effective,
    };
    submit_pay_rate(&data, &session, &lang, auth.bearer, new_rate).await
}


// ---------------------------------------------------------------------------
// Budget allocations (org tiers)
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
pub struct BudgetForm {
    pub csrf_token: String,
    pub amount: String,
    /// Starting year of the fiscal year the envelope applies to; the options
    /// come from the API's own labels, so the April-1 rule lives only there.
    pub fiscal_year: String,
}

#[post("/{lang}/org_tier/{org_tier_id}/budget")]
pub async fn set_org_tier_budget_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<(String, String)>,
    form: web::Form<BudgetForm>,
    req: HttpRequest,
) -> impl Responder {
    let (lang, org_tier_id) = path_params.into_inner();
    let session = req.get_session();

    let auth = match security::require_role(&session, &lang, MinimumRole::Operator) {
        Ok(auth) => auth,
        Err(response) => return response,
    };

    if !security::verify_csrf_token(&session, &form.csrf_token) {
        csrf_failure_flash(&session, &lang);
        return redirect_to(format!("/{}/org_tier/{}", &lang, &org_tier_id));
    }

    let Some(cents) = dollars_to_cents(&form.amount, &lang) else {
        security::add_flash(&session, "danger", by_lang(&lang,
            "Enter a valid allocation amount.",
            "Entrez un montant d'allocation valide."));
        return redirect_to(format!("/{}/org_tier/{}", &lang, &org_tier_id));
    };
    let Some(fiscal_year) = form.fiscal_year.trim().parse::<i64>().ok().filter(|y| (2000..2100).contains(y)) else {
        security::add_flash(&session, "danger", by_lang(&lang,
            "Choose a valid fiscal year.",
            "Choisissez un exercice financier valide."));
        return redirect_to(format!("/{}/org_tier/{}", &lang, &org_tier_id));
    };

    match set_budget_allocation(org_tier_id.clone(), Some(fiscal_year), cents, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success",
            by_lang(&lang, "Budget allocation saved.", "Allocation budgétaire enregistrée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/org_tier/{}", &lang, &org_tier_id))
}
