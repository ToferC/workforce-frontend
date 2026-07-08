use actix_web::{web, get, HttpResponse, HttpRequest, Responder};

/// Flip the language prefix on any app path and redirect there. The header's
/// toggle links to `/toggle_language{current path}`, so this must accept a
/// tail of any depth — fixed-depth routes broke the toggle on deep pages like
/// `/{lang}/role/{id}/requirement/{rid}/edit`.
#[get("/toggle_language/{lang}{tail:.*}")]
pub async fn toggle_language(
    _path: web::Path<(String, String)>,
    req: HttpRequest,
) -> impl Responder {
    // Work from the raw (still percent-encoded) request path so encoded
    // segments round-trip into the redirect untouched.
    let rest = req
        .uri()
        .path()
        .strip_prefix("/toggle_language/")
        .unwrap_or("");
    let (current_lang, tail) = match rest.split_once('/') {
        Some((lang, tail)) => (lang, Some(tail)),
        None => (rest, None),
    };

    let new_lang = match current_lang {
        "en" => "fr",
        _ => "en",
    };

    let location = match tail {
        Some(tail) if !tail.is_empty() => format!("/{}/{}", new_lang, tail),
        _ => format!("/{}", new_lang),
    };

    HttpResponse::Found()
        .append_header(("Location", location))
        .finish()
}
