// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs

use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, HttpMessage, Responder, get, post, web};
use actix_session::{SessionExt};
use actix_identity::{Identity};

use crate::{AppData, generate_basic_context, graphql};

use super::LoginForm;

#[get("/{lang}/log_in")]
pub async fn login_handler(
    path: web::Path<String>,
    data: web::Data<AppData>,
    
    req:HttpRequest,
    id: Option<Identity>,
) -> impl Responder {

    let lang = path.into_inner();

    let session = req.get_session();

    let ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let rendered = data.tmpl.render("authentication/log_in.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/log_in")]
pub async fn login_form_input(
    path: web::Path<String>,
    data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<LoginForm>,
    _id: Option<Identity>,
) -> impl Responder {

    let lang = path.into_inner();

    // validate form has data or re-load form
    if form.email.is_empty() || form.password.is_empty() {
        println!("Form is empty");
        return HttpResponse::Found().append_header(("Location", format!("/{}/log_in", &lang))).finish()
    };
    
    let login_data = graphql::login(
        form.email.to_lowercase().trim().to_string(),
        form.password.clone(), 
        &data.api_url,
        Arc::clone(&data.client),
    )
        .await
        .expect("Unable to login").sign_in;

    // Add user_name and role to session
    Identity::login(&req.extensions(), login_data.email.to_owned())
        .expect("Unable to login / identity");

    println!("{:?}", &login_data);

    let session = req.get_session();

    // The API stores roles in uppercase ("ADMIN"); normalize so template
    // checks like role == "admin" and handler guards compare consistently
    session.insert("role", login_data.role.to_lowercase())
        .expect("Unable to set role");

    session.insert("user_id", login_data.id.to_owned())
        .expect("Unable to set user_id");

    session.insert("session_user", login_data.email.to_owned())
        .expect("Unable to set user name");

    session.insert("bearer", login_data.bearer.to_owned())
        .expect("Unable to set bearer");

    // Store session expiration time as ISO string

    session.insert("expires_at", login_data.expires_at.to_string())
        .expect("Unable to set expires_at");
    

    return HttpResponse::Found()
        .append_header(("Location", "/"))
        .append_header(("Bearer", login_data.bearer))
        .finish()
}

#[get("/{lang}/log_out")]
pub async fn logout(
    path: web::Path<String>,
    _data: web::Data<AppData>,
    req: HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    let lang = path.into_inner();

    let session = req.get_session();

    session.clear();
    id.unwrap().logout();

    HttpResponse::Found().append_header(("Location", format!("/{}", &lang))).finish()
}