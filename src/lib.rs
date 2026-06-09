pub mod models;
pub mod handlers;
pub mod graphql;
pub mod errors;

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

    ctx
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

