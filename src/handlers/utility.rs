use serde::Deserialize;
use actix_web::{web, get, HttpResponse, HttpRequest, Responder};

#[derive(Deserialize, Debug)]
pub struct UrlParams {
    pub lang: Option<String>,
}

#[get("/toggle_language/{lang}")]
pub async fn toggle_language_index(
    path: web::Path<String>,
) -> impl Responder {
    let lang = path.into_inner();
    let new_lang = match lang.as_str() {
        "fr" => "en",
        "en" => "fr",
        _ => "en",
    };

    println!("New lang: {}", &new_lang);

    HttpResponse::Found()
        .header("Accept-Language", new_lang)
        .header("Location", format!("/{}", &new_lang))
        .finish()
}

#[get("/toggle_language/{lang}/{url}")]
pub async fn toggle_language(
    path: web::Path<(String, String)>,
    _req: HttpRequest,
) -> impl Responder {
    let (lang, url) = path.into_inner();
    println!("url: {}", &url);

    let new_lang = if lang.as_str() == "en" {
        "fr"
    } else {
        "en"
    };

    println!("New lang: {}", &new_lang);

    // Remove leading character "/"
    let cleaned_url: &str = url.split("/").into_iter().last().expect("Unable to find url");

    HttpResponse::Found()
        .header("Location", format!("/{}/{}", &new_lang, &cleaned_url))
        .finish()
}

#[get("/toggle_language/{lang}/{url}/{url2}")]
pub async fn toggle_language_two(
    path: web::Path<(String, String, String)>,
    _req: HttpRequest,
) -> impl Responder {
    let (lang, url, url2) = path.into_inner();
    println!("url: {}/{}", &url, &url2);

    let new_lang = if lang.as_str() == "en" {
        "fr"
    } else {
        "en"
    };

    println!("New lang: {}", &new_lang);

    HttpResponse::Found()
        .header("Location", format!("/{}/{}/{}", &new_lang, &url, &url2))
        .finish()
}

#[get("/toggle_language/{lang}/{url}/{url2}/{url3}")]
pub async fn toggle_language_three(
    path: web::Path<(String, String, String, String)>,
    _req: HttpRequest,
) -> impl Responder {
    let (lang, url, url2, url3) = path.into_inner();
    println!("url: {}/{}/{}", &url, &url2, &url3);

    let new_lang = if lang.as_str() == "en" {
        "fr"
    } else {
        "en"
    };

    println!("New lang: {}", &new_lang);

    HttpResponse::Found()
        .header("Location", format!("/{}/{}/{}/{}", &new_lang, &url, &url2, &url3))
        .finish()
}