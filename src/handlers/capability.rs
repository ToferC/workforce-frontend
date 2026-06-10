use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use actix_identity::Identity;
use std::sync::Arc;
use crate::{AppData, generate_basic_context};
use crate::graphql::get_capability_by_name_and_level;

#[get("/{lang}/capability_search/{name}/{level}")]
pub async fn capability_search(
    path_params: web::Path<(String, String, String)>,
    data: web::Data<AppData>,
    req: HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    let (lang, name, level) = path_params.into_inner();
    println!("CALL CAPABILITY SEARCH");

    let session = req.get_session();
    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };
    
    // query graphql API
    let results = get_capability_by_name_and_level(
        name.to_lowercase().trim().to_string(),
        level.clone(),
        bearer.clone(),
        &data.api_url,
        Arc::clone(&data.client),
    )
    .await
    .expect("Unable to find capabilities");

    println!("{:?}", &results);
             
    ctx.insert("capabilities", &results.capabilities_by_name_and_level);
    ctx.insert("name", &name.to_owned());
    ctx.insert("level", &level);

    let rendered = data.tmpl.render("capability/capability_search_results.html", &ctx).unwrap();
    HttpResponse::Ok()
        .header("Bearer", bearer)
        .body(rendered)
}