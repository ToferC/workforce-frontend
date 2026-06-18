use actix_session::SessionExt;
use actix_web::{web, get, Responder, HttpResponse, HttpRequest};
use actix_identity::Identity;

use std::sync::Arc;
use crate::{generate_basic_context, AppData, graphql::all_organizations};
use crate::security;

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

    let bearer = match session.get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    match all_organizations(bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(r) => {
            ctx.insert("organizations", &r.all_organizations);
        }
        Err(e) => {
            let err_msg = format!("{}", e);
            if err_msg.contains("ExpiredSignature")
                || err_msg.contains("InvalidToken")
                || err_msg.contains("Access denied")
                || bearer_is_empty_or_missing(&session)
            {
                session.clear();
                security::add_flash(
                    &session,
                    "warning",
                    "Your session has expired. Please log in again.",
                );
                return HttpResponse::Found()
                    .append_header(("Location", format!("/{}/log_in", lang)))
                    .finish();
            }
            ctx.insert("organizations", &Vec::<String>::new());
        }
    }

    let rendered = data.tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

fn bearer_is_empty_or_missing(session: &actix_session::Session) -> bool {
    match session.get::<String>("bearer") {
        Ok(Some(b)) => b.is_empty(),
        _ => true,
    }
}