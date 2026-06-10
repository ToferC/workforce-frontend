use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use actix_identity::{Identity};


use std::sync::Arc;
use crate::{AppData, generate_basic_context};
use crate::graphql::{get_publication_by_id};

#[get("/{lang}/publication/{publication_id}")]
pub async fn publication_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req:HttpRequest) -> impl Responder {
    let (lang, publication_id) = path_params.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_publication_by_id(publication_id, bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get people");

    ctx.insert("publication", &r.publication_by_id);

    let rendered = data.tmpl.render("publication/publication.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}