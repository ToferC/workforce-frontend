// Render the entity templates with realistic contexts to catch Tera runtime
// errors (bad macro arguments, undefined variables) that template parsing
// at startup does not.

use tera::{Tera, Context};
use fluent_templates::{FluentLoader, static_loader};
use serde_json::json;

use frontend::security::FlashMessage;

static_loader! {
    static LOCALES = {
        locales: "./i18n/",
        fallback_language: "en",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

fn tera() -> Tera {
    let mut tera = Tera::new("templates/**/*").unwrap();
    tera.register_filter("snake_case", tera_text_filters::snake_case);
    tera.register_function("fluent", FluentLoader::new(&*LOCALES));
    tera
}

// Mirrors what generate_basic_context inserts for every page
fn base_context(lang: &str, role: &str) -> Context {
    let mut ctx = Context::new();
    ctx.insert("lang", lang);
    ctx.insert("path", "/");
    ctx.insert("session_user", "tester@example.com");
    ctx.insert("role", role);
    ctx.insert("user_id", "test-user-id");
    ctx.insert("expires_at", "2099-01-01 00:00:00");
    ctx.insert("flash_messages", &vec![FlashMessage {
        level: "success".to_string(),
        message: "Test flash".to_string(),
    }]);
    ctx.insert("csrf_token", "test-csrf-token");
    ctx
}

fn sample_organization() -> serde_json::Value {
    json!({
        "id": "11111111-1111-1111-1111-111111111111",
        "nameEn": "Test Organization",
        "nameFr": "Organisation test",
        "acronymEn": "TO",
        "acronymFr": "OT",
        "orgType": "department",
        "url": "https://example.com",
        "retiredAt": null,
        "topOrgTier": [],
        "capabilityCounts": [],
        "publications": [],
        "affiliations": [],
    })
}

#[test]
fn organization_form_renders_for_create() {
    let tera = tera();
    for lang in ["en", "fr"] {
        let mut ctx = base_context(lang, "operator");
        ctx.insert("edit", &false);
        ctx.insert("organization", &json!({
            "nameEn": "", "nameFr": "", "acronymEn": "", "acronymFr": "",
            "orgType": "", "url": "",
        }));
        let html = tera.render("organization/organization_form.html", &ctx).unwrap();
        assert!(html.contains("/organization/new"));
        assert!(html.contains("name=\"csrf_token\" value=\"test-csrf-token\""));
    }
}

#[test]
fn organization_form_renders_for_edit() {
    let tera = tera();
    for lang in ["en", "fr"] {
        let mut ctx = base_context(lang, "operator");
        ctx.insert("edit", &true);
        ctx.insert("organization", &sample_organization());
        let html = tera.render("organization/organization_form.html", &ctx).unwrap();
        assert!(html.contains("/organization/11111111-1111-1111-1111-111111111111/edit"));
        assert!(html.contains("value=\"Test Organization\""));
        assert!(html.contains("value=\"Organisation test\""));
    }
}

#[test]
fn organization_retire_page_renders() {
    let tera = tera();
    for lang in ["en", "fr"] {
        let mut ctx = base_context(lang, "operator");
        ctx.insert("organization", &sample_organization());
        let html = tera.render("organization/organization_retire.html", &ctx).unwrap();
        assert!(html.contains("/organization/11111111-1111-1111-1111-111111111111/retire"));
    }
}

#[test]
fn organization_detail_shows_actions_for_operator_only() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("organization", &sample_organization());
    let html = tera.render("organization/organization.html", &ctx).unwrap();
    assert!(html.contains("/organization/11111111-1111-1111-1111-111111111111/edit"));
    assert!(html.contains("/organization/11111111-1111-1111-1111-111111111111/retire"));

    let mut ctx = base_context("en", "user");
    ctx.insert("organization", &sample_organization());
    let html = tera.render("organization/organization.html", &ctx).unwrap();
    assert!(!html.contains("/edit"));
    assert!(!html.contains("/retire"));
}

#[test]
fn organization_detail_hides_retire_when_already_retired() {
    let tera = tera();
    let mut ctx = base_context("en", "admin");
    let mut org = sample_organization();
    org["retiredAt"] = json!("2026-01-01T00:00:00");
    ctx.insert("organization", &org);
    let html = tera.render("organization/organization.html", &ctx).unwrap();
    assert!(html.contains("/edit"));
    assert!(!html.contains("/retire"));
}

#[test]
fn index_shows_new_organization_button_for_operator_only() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("organizations", &json!([
        {"id": "11111111-1111-1111-1111-111111111111", "nameEn": "Test Organization"}
    ]));
    let html = tera.render("index.html", &ctx).unwrap();
    assert!(html.contains("/en/organization/new"));

    let mut ctx = base_context("en", "user");
    ctx.insert("organizations", &json!([]));
    let html = tera.render("index.html", &ctx).unwrap();
    assert!(!html.contains("/en/organization/new"));
}
