use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use actix_identity::{Identity};


use crate::{AppData, generate_basic_context};
use crate::graphql::{get_organization_by_id};

#[get("/{lang}/organization/{organization_id}")]
pub async fn organization_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path_params: web::Path<(String, String)>,

    req:HttpRequest) -> impl Responder {
    let (lang, organization_id) = path_params.into_inner();
    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_organization_by_id(organization_id, bearer, &data.api_url)
        .expect("Unable to get people");

    ctx.insert("organization", &r.organization_by_id);

    let rendered = data.tmpl.render("organization/organization.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}