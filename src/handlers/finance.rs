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

#[derive(Deserialize, Debug)]
pub struct PayRateForm {
    pub csrf_token: String,
    pub occupational_group: String,
    pub occupational_level: String,
    pub rank: String,
    pub annual_rate: String,
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

    // Split civilian and military rates for the two tables.
    let mut civilian: Vec<serde_json::Value> = Vec::new();
    let mut military: Vec<serde_json::Value> = Vec::new();
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

#[post("/{lang}/admin/pay_rates/new")]
pub async fn pay_rate_create_post(
    data: web::Data<AppData>,
    _id: Option<Identity>,
    path_params: web::Path<String>,
    form: web::Form<PayRateForm>,
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

    let has_civilian = !form.occupational_group.trim().is_empty();
    let has_military = !form.rank.trim().is_empty();
    let cents = dollars_to_cents(&form.annual_rate, &lang);
    let effective = parse_form_date(&form.effective_date);

    let (Some(cents), Some(effective)) = (cents, effective) else {
        security::add_flash(&session, "danger", by_lang(&lang,
            "Enter a valid annual rate and effective date.",
            "Entrez un taux annuel et une date d'entrée en vigueur valides."));
        return redirect_to(format!("/{}/admin/pay_rates", &lang));
    };
    if has_civilian == has_military {
        security::add_flash(&session, "danger", by_lang(&lang,
            "Choose either an occupational group and level, or a rank.",
            "Choisissez soit un groupe professionnel et un niveau, soit un grade."));
        return redirect_to(format!("/{}/admin/pay_rates", &lang));
    }

    let new_rate = create_pay_rate::NewPayRate {
        occupational_group: if has_civilian {
            serde_json::from_value(json!(form.occupational_group)).ok()
        } else { None },
        occupational_level: if has_civilian {
            form.occupational_level.trim().parse::<i64>().ok()
        } else { None },
        rank: if has_military {
            serde_json::from_value(json!(form.rank)).ok()
        } else { None },
        annual_rate_cents: cents,
        effective_date: effective,
    };

    match create_pay_rate(new_rate, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success",
            by_lang(&lang, "Pay rate added.", "Taux de rémunération ajouté.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/admin/pay_rates", &lang))
}


// ---------------------------------------------------------------------------
// Budget allocations (org tiers)
// ---------------------------------------------------------------------------

/// Starting year of the current fiscal year (2026 = FY 2026-27).
fn current_fiscal_year_start() -> i64 {
    let today = chrono::Utc::now().date_naive();
    use chrono::Datelike;
    if today.month() >= 4 { today.year() as i64 } else { today.year() as i64 - 1 }
}

#[derive(Deserialize, Debug)]
pub struct BudgetForm {
    pub csrf_token: String,
    pub amount: String,
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

    match set_budget_allocation(org_tier_id.clone(), current_fiscal_year_start(), cents, auth.bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(_) => security::add_flash(&session, "success",
            by_lang(&lang, "Budget allocation saved.", "Allocation budgétaire enregistrée.")),
        Err(e) => security::add_flash(&session, "danger", &e.to_string()),
    };

    redirect_to(format!("/{}/org_tier/{}", &lang, &org_tier_id))
}
