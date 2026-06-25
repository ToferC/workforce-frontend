
use actix_session::SessionExt;
use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use actix_identity::Identity;
use crate::{AppData, generate_basic_context};


pub async fn f404(
    path: web::Path<String>,
    data: web::Data<AppData>,

    req:HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let uri_path = req.uri().path();
    ctx.insert("path", &uri_path);

    let rendered = data.tmpl.render("errors/404.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// Catch-all fallback for any request that matches no route. Wired as the
/// App's `default_service`, it renders the friendly 404 page (which links back
/// to the index) with a real 404 status instead of Actix's plain-text default.
pub async fn default_404(
    data: web::Data<AppData>,
    req: HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    let session = req.get_session();
    let uri_path = req.uri().path().to_string();

    // Keep the page bilingual by reading the language from the first path
    // segment (e.g. /fr/does-not-exist), defaulting to English otherwise.
    let lang = match uri_path.split('/').nth(1) {
        Some("fr") => "fr",
        _ => "en",
    }
    .to_string();

    let mut ctx = generate_basic_context(id, &lang, &uri_path, &session);
    ctx.insert("path", &uri_path);

    let rendered = data.tmpl.render("errors/404.html", &ctx).unwrap();
    HttpResponse::NotFound()
        .content_type("text/html; charset=utf-8")
        .body(rendered)
}

#[get("/{lang}/not_found")]
pub async fn not_found(
    path: web::Path<String>,
    data: web::Data<AppData>,

    req:HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();
    let ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let rendered = data.tmpl.render("errors/not_found.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/internal_server_error")]
pub async fn internal_server_error(
    path: web::Path<String>,
    data: web::Data<AppData>,

    req: HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();
    let ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let rendered = data.tmpl.render("errors/internal_server_error.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/not_authorized")]
pub async fn not_authorized(
    path: web::Path<String>,
    data: web::Data<AppData>,

    req:HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();
    let ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let rendered = data.tmpl.render("errors/not_authorized.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}
