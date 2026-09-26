pub mod models;
pub mod handlers;
pub mod graphql;
pub mod errors;
pub mod security;
pub mod notifications;

use tera::{Tera, Context};
use actix_identity::Identity;
use actix_session::Session;
use reqwest::Client;
use std::sync::Arc;


extern crate strum;
#[macro_use]
extern crate strum_macros;

const APP_NAME: &str = "Workforce-frontend";

use fluent_templates::{langid, static_loader, LanguageIdentifier, Loader};

// Fluent bundles for EN/FR, shared by the Tera `fluent` function and
// server-side label lookups.
static_loader! {
    pub static LOCALES = {
        locales: "./i18n/",
        fallback_language: "en",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

#[derive(Clone, Debug)]
pub struct AppData {
    pub tmpl: Tera,
    pub api_url: String,
    pub client: Arc<Client>,
}

/// Generate context, session_user, role and node_names from id and lang
pub fn generate_basic_context(
    identity: Option<Identity>,
    lang: &str,
    path: &str,
    session: &Session,
) -> (Context) 
{    
    let mut ctx = Context::new();

    let session_user = match identity {
        Some(i) => i.id().unwrap(),
        None => "".to_string(),
    };

    // Get session data and add to context
    println!("Getting Session data and adding to Context");

    let (role, user_id, expires_at) = extract_session_data(session);

    ctx.insert("session_user", &session_user);
    ctx.insert("role", &role);
    ctx.insert("user_id", &user_id);
    ctx.insert("expires_at", &expires_at);

    let validated_lang = match lang {
        "fr" => "fr",
        "en" => "en",
        _ => "en",
    };

    ctx.insert("lang", &validated_lang);
    ctx.insert("path", &path);

    // One-time flash messages and the CSRF token for any forms on the page
    ctx.insert("flash_messages", &security::take_flash(session));
    ctx.insert("csrf_token", &security::get_or_create_csrf_token(session));

    ctx
}

/// Pick a string by request language. Used for server-generated text like
/// flash messages, which can't go through the Tera Fluent filter.
pub fn by_lang<'a>(lang: &str, en: &'a str, fr: &'a str) -> &'a str {
    if lang == "fr" { fr } else { en }
}

/// Whole-dollar money string from integer cents: "$1,234,567" in English,
/// "1 234 567 $" in French (non-breaking spaces). Cents are truncated —
/// salary and contract figures read better without them.
pub fn format_cents(cents: i64, lang: &str) -> String {
    let dollars = cents / 100;
    let negative = dollars < 0;
    let digits = dollars.abs().to_string();
    let sep = if lang == "fr" { '\u{a0}' } else { ',' };
    let mut grouped = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            grouped.push(sep);
        }
        grouped.push(c);
    }
    let sign = if negative { "-" } else { "" };
    if lang == "fr" {
        format!("{}{}\u{a0}$", sign, grouped)
    } else {
        format!("{}${}", sign, grouped)
    }
}

/// Compact money for at-a-glance tiles: "$663.4M" / "663,4 M$", "$12.9K",
/// whole dollars below 1,000.
pub fn format_cents_compact(cents: i64, lang: &str) -> String {
    let dollars = cents as f64 / 100.0;
    let (scaled, suffix) = match dollars.abs() {
        d if d >= 1e9 => (dollars / 1e9, if lang == "fr" { "\u{a0}G" } else { "B" }),
        d if d >= 1e6 => (dollars / 1e6, if lang == "fr" { "\u{a0}M" } else { "M" }),
        d if d >= 1e3 => (dollars / 1e3, if lang == "fr" { "\u{a0}k" } else { "K" }),
        _ => return format_cents(cents, lang),
    };
    let number = format!("{:.1}", scaled);
    if lang == "fr" {
        format!("{}{}\u{a0}$", number.replace('.', ","), suffix)
    } else {
        let (sign, number) = number.strip_prefix('-').map_or(("", number.as_str()), |n| ("-", n));
        format!("{}${}{}", sign, number, suffix)
    }
}

/// Tera filter over `format_cents`: `{{ summary.budgetedCents | money(lang=lang) }}`;
/// `compact=true` gives the tile form ("$663.4M").
pub fn money_filter(
    value: &tera::Value,
    args: &std::collections::HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let cents = value
        .as_i64()
        .or_else(|| value.as_f64().map(|f| f as i64))
        .unwrap_or(0);
    let lang = args.get("lang").and_then(|v| v.as_str()).unwrap_or("en");
    let compact = args.get("compact").and_then(|v| v.as_bool()).unwrap_or(false);
    Ok(tera::Value::String(if compact { format_cents_compact(cents, lang) } else { format_cents(cents, lang) }))
}

/// Numeric weight for each CapabilityLevel; shared by analytics and org chart.
pub fn level_weight(level: &str) -> i64 {
    match level {
        "DESIRED"     => 1,
        "NOVICE"      => 2,
        "EXPERIENCED" => 3,
        "EXPERT"      => 4,
        "SPECIALIST"  => 5,
        _             => 0,
    }
}

/// How well a set of capabilities covers a role's requirements, matched by
/// skill name. The held level is the validated level when there is one,
/// otherwise the self-identified level. Shared by the person page's current
/// roles and job matches so both read the same way:
/// `{rows: [{name, domain, required, held, validated, met}], met, total, pct}`.
pub fn requirement_fit(requirements: &serde_json::Value, capabilities: &serde_json::Value) -> serde_json::Value {
    let caps = capabilities.as_array().map(Vec::as_slice).unwrap_or_default();
    let rows: Vec<serde_json::Value> = requirements.as_array().into_iter().flatten().map(|req| {
        let name = req["nameEn"].as_str().unwrap_or("");
        let required = req["requiredLevel"].as_str().unwrap_or("");
        let cap = caps.iter().find(|c| c["nameEn"].as_str() == Some(name));
        let validated = cap.and_then(|c| c["validatedLevel"].as_str());
        let held = validated.or_else(|| cap.and_then(|c| c["selfIdentifiedLevel"].as_str()));
        serde_json::json!({
            "name": name,
            "domain": req["domain"],
            "required": required,
            "held": held,
            "validated": validated.is_some(),
            "met": held.map_or(false, |h| level_weight(h) >= level_weight(required)),
        })
    }).collect();
    let total = rows.len();
    let met = rows.iter().filter(|r| r["met"] == true).count();
    let pct = if total == 0 { 100 } else { met * 100 / total };
    serde_json::json!({"rows": rows, "met": met, "total": total, "pct": pct})
}

/// Short, localized label for an API enum value — the Rust twin of
/// `labels::enum_label` in templates/macros/labels.html, reading the same
/// `enum-<kind>-<value>` Fluent keys (value lower-cased, `_` → `-`). Used where
/// labels are built server-side (chart series, org-chart chips). A missing key
/// falls back to sentence case, never ALL_CAPS.
pub fn enum_label(kind: &str, value: &str, lang: &str) -> String {
    let key = format!("enum-{}-{}", kind, value.to_lowercase().replace('_', "-"));
    let lang_id: LanguageIdentifier = lang.parse().unwrap_or_else(|_| langid!("en"));
    LOCALES.try_lookup(&lang_id, &key).unwrap_or_else(|| {
        let mut words = value.replace('_', " ").to_lowercase();
        if let Some(first) = words.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        words
    })
}

/// Hex colour for a WorkStatus, used for chart fills.
pub fn status_color(status: &str) -> &'static str {
    match status {
        "PLANNING"    => "#6c757d",
        "IN_PROGRESS" => "#0d6efd",
        "COMPLETED"   => "#198754",
        "BLOCKED"     => "#dc3545",
        "CANCELLED"   => "#adb5bd",
        _             => "#6c757d",
    }
}

/// Serialize a chart option to a JSON string safe to embed inside an inline
/// `<script type="application/json">` block. Replaces every `<` with its JSON
/// unicode escape so that a literal `</script>` inside user-controlled strings
/// (names, work descriptions, requirement titles) cannot terminate the script
/// element early — which would both break the chart and allow stored
/// HTML/script injection. `JSON.parse` decodes the escape back to `<`, so chart
/// labels still render correctly.
pub fn chart_json(value: &serde_json::Value) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|_| "{}".to_string())
        .replace('<', "\\u003c")
}

/// CSS group name for a SkillDomain key (maps to .domain-{group} CSS class).
pub fn domain_group(key: &str) -> &'static str {
    match key {
        "COMBAT" | "INTELLIGENCE" | "STRATEGY" | "JOINT_OPERATIONS" => "ops",
        "ENGINEERING" | "MEDICAL" => "science",
        "SOFTWARE_ENGINEERING" | "CLOUD_PLATFORM_DEV_OPS"
        | "DATA_ANALYTICS_AND_AI" | "CYBER_SECURITY" | "USER_EXPERIENCE" => "digital",
        "PRODUCT_AGILE_AND_DELIVERY" | "PROCUREMENT_AND_VENDOR_MANAGEMENT" => "delivery",
        "PEOPLE_AND_ORGANISATIONAL_LEADERSHIP" | "GOVERNANCE" | "CORPORATE_SERVICES" => "corp",
        _ => "secondary",
    }
}

pub fn extract_session_data(session: &Session) -> (String, String, String) {

    let role_data = session.get::<String>("role");

    let role = match role_data {
        Ok(Some(r)) => r,
        Ok(None) => "".to_string(),
        Err(_) => "".to_string(),
    };

    let id_data = session.get::<String>("user_id");

    let user_id = match id_data {
        Ok(Some(u)) => u,
        Ok(None) => "".to_string(),
        Err(_) => "".to_string(),
    };

    let expires_at_data = session.get::<String>("expires_at");

    let expires_at = match expires_at_data {
        Ok(Some(e)) => e,
        Ok(None) => "".to_string(),
        Err(_) => "".to_string(),
    };

    println!("{}-{}", &role, &user_id);

    (role, user_id, expires_at)
}

