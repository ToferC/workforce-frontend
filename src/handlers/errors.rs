
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
