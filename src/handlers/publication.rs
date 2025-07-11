use actix_session::UserSession;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use actix_identity::{Identity};


use crate::{AppData, generate_basic_context};
use crate::graphql::{get_publication_by_id};

#[get("/{lang}/publication/{publication_id}")]
pub async fn publication_by_id(
    data: web::Data<AppData>,
    id: Identity,
    web::Path((lang, publication_id)): web::Path<(String, String)>,
    
    req:HttpRequest) -> impl Responder {

    let (mut ctx, _user, _lang, _path) = generate_basic_context(id, &lang, req.uri().path());

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_publication_by_id(publication_id, bearer, &data.api_url)
        .expect("Unable to get people");

    ctx.insert("publication", &r.publication_by_id);

    let rendered = data.tmpl.render("publication/publication.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}