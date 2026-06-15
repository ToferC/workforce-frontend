pub mod models;
pub mod handlers;
pub mod graphql;
pub mod errors;
pub mod security;

use actix_web::Error;
use tera::{Tera, Context};
use actix_identity::Identity;
use actix_session::Session;
use reqwest::Client;
use std::sync::Arc;


extern crate strum;
#[macro_use]
extern crate strum_macros;

const APP_NAME: &str = "Workforce-frontend";

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

/// Short display label for a SkillDomain key.
pub fn domain_short_label(key: &str) -> &'static str {
    match key {
        "COMBAT"                                => "Combat",
        "INTELLIGENCE"                          => "Intelligence",
        "STRATEGY"                              => "Strategy",
        "ENGINEERING"                           => "Engineering",
        "MEDICAL"                               => "Medical",
        "JOINT_OPERATIONS"                      => "Joint Ops",
        "SOFTWARE_ENGINEERING"                  => "Software Eng",
        "CLOUD_PLATFORM_DEV_OPS"               => "Cloud/DevOps",
        "DATA_ANALYTICS_AND_AI"                => "Data & AI",
        "CYBER_SECURITY"                        => "Cyber",
        "PRODUCT_AGILE_AND_DELIVERY"           => "Product/Agile",
        "USER_EXPERIENCE"                       => "UX",
        "PROCUREMENT_AND_VENDOR_MANAGEMENT"    => "Procurement",
        "PEOPLE_AND_ORGANISATIONAL_LEADERSHIP" => "People & Org",
        "GOVERNANCE"                            => "Governance",
        "CORPORATE_SERVICES"                    => "Corporate",
        _                                       => "—",
    }
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

