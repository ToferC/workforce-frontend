use actix_session::SessionExt;
use actix_web::{web, get, Responder, HttpResponse, HttpRequest};
use actix_identity::Identity;

use std::sync::Arc;
use crate::{generate_basic_context, AppData, graphql::all_organizations};

#[get("/")]
pub async fn raw_index() -> impl Responder {
    return HttpResponse::Found().header("Location", "/en").finish()
}

#[get("/{lang}")]
pub async fn index(
    data: web::Data<AppData>,
    params: web::Path<String>,

    id: Option<Identity>,
    req: HttpRequest,
) -> impl Responder {

    let lang = params.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = all_organizations(bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get people");

    ctx.insert("organizations", &r.all_organizations);

    let rendered = data.tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}