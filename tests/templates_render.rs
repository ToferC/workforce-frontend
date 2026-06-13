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

fn sample_org_tier() -> serde_json::Value {
    json!({
        "id": "22222222-2222-2222-2222-222222222222",
        "nameEn": "Test Tier",
        "nameFr": "Niveau test",
        "tierLevel": 2,
        "primaryDomain": "STRATEGY",
        "retiredAt": null,
        "organization": {"id": "11111111-1111-1111-1111-111111111111", "nameEn": "Test Organization"},
        "parentOrganizationTier": {"id": "33333333-3333-3333-3333-333333333333", "nameEn": "Parent Tier"},
        "owner": {"id": "44444444-4444-4444-4444-444444444444", "givenName": "Jane", "familyName": "Doe", "email": "jane@example.com"},
        "childOrganizationTier": [
            {"id": "55555555-5555-5555-5555-555555555555", "nameEn": "Child Tier", "nameFr": "Niveau enfant", "tierLevel": 3, "retiredAt": null}
        ],
        "teams": [{
            "id": "66666666-6666-6666-6666-666666666666",
            "nameEnglish": "Test Team",
            "nameFrench": "Équipe test",
            "owner": {"id": "44444444-4444-4444-4444-444444444444", "givenName": "Jane", "familyName": "Doe"},
            "occupiedRoles": [{
                "id": "77777777-7777-7777-7777-777777777777",
                "titleEnglish": "Analyst", "titleFrench": "Analyste",
                "person": {"id": "88888888-8888-8888-8888-888888888888", "givenName": "Sam", "familyName": "Lee"}
            }],
            "vacantRoles": [{
                "id": "99999999-9999-9999-9999-999999999999",
                "titleEnglish": "Advisor", "titleFrench": "Conseiller"
            }],
        }],
    })
}

fn domain_options() -> serde_json::Value {
    json!([{"value": "STRATEGY", "label": "Strategy"}, {"value": "MEDICAL", "label": "Medical"}])
}

fn parent_options() -> serde_json::Value {
    json!([{"value": "33333333-3333-3333-3333-333333333333", "label": "Parent Tier (level 1)"}])
}

#[test]
fn org_tier_form_renders_for_create_and_edit() {
    let tera = tera();
    for (lang, edit) in [("en", false), ("fr", false), ("en", true), ("fr", true)] {
        let mut ctx = base_context(lang, "operator");
        ctx.insert("edit", &edit);
        ctx.insert("org_tier", &sample_org_tier());
        ctx.insert("skill_domains", &domain_options());
        ctx.insert("parent_tier_options", &parent_options());
        let html = tera.render("org_tier/org_tier_form.html", &ctx).unwrap();
        if edit {
            assert!(html.contains("/org_tier/22222222-2222-2222-2222-222222222222/edit"));
            // current parent pre-selected
            assert!(html.contains("value=\"33333333-3333-3333-3333-333333333333\" selected"));
        } else {
            assert!(html.contains("/org_tier/new"));
        }
        assert!(html.contains("name=\"organization_id\""));
        assert!(html.contains("STRATEGY"));
    }
}

#[test]
fn org_tier_retire_page_renders() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("org_tier", &sample_org_tier());
    let html = tera.render("org_tier/org_tier_retire.html", &ctx).unwrap();
    assert!(html.contains("/org_tier/22222222-2222-2222-2222-222222222222/retire"));
}

#[test]
fn org_chart_builder_page_renders() {
    let tera = tera();
    for role in ["operator", "user"] {
        let mut ctx = base_context("en", role);
        ctx.insert("organization", &sample_organization());
        ctx.insert("organization_id", "11111111-1111-1111-1111-111111111111");
        ctx.insert("root_tiers", &json!([{
            "id": "22222222-2222-2222-2222-222222222222",
            "nameEn": "Test Tier", "nameFr": "Niveau test", "tierLevel": 1, "retiredAt": null,
        }]));
        ctx.insert("tier_count", &1);
        let html = tera.render("org_chart/builder.html", &ctx).unwrap();
        // node lazy-loads on expand
        assert!(html.contains("hx-get=\"/en/org_tier/22222222-2222-2222-2222-222222222222/node\""));
        assert!(html.contains("id=\"info-panel\""));
        let has_add = html.contains("/en/org_tier/new?organization=");
        assert_eq!(has_add, role == "operator");
    }
}

#[test]
fn org_chart_node_partial_renders() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("node", &sample_org_tier());
    let html = tera.render("org_chart/node.html", &ctx).unwrap();
    // child tier rendered as a lazy expandable node
    assert!(html.contains("node-body-55555555-5555-5555-5555-555555555555"));
    // team with occupied and vacant roles
    assert!(html.contains("Test Team"));
    assert!(html.contains("Sam Lee"));
    assert!(html.contains("Advisor"));
    // operator can add a child tier
    assert!(html.contains("parent=22222222-2222-2222-2222-222222222222"));
}

#[test]
fn org_chart_panel_partial_renders() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("org_tier", &sample_org_tier());
    let html = tera.render("org_chart/panel.html", &ctx).unwrap();
    assert!(html.contains("Jane Doe"));
    assert!(html.contains("/org_tier/22222222-2222-2222-2222-222222222222/edit"));
}

#[test]
fn org_chart_add_tier_form_partial_renders() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("org_tier", &sample_org_tier());
    ctx.insert("skill_domains", &domain_options());
    let html = tera.render("org_chart/add_tier_form.html", &ctx).unwrap();
    assert!(html.contains("hx-post=\"/en/org_tier/new\""));
    assert!(html.contains("name=\"csrf_token\""));
    // parent passed through as hidden field
    assert!(html.contains("name=\"parent_tier\" value=\"33333333-3333-3333-3333-333333333333\""));
}

fn sample_team() -> serde_json::Value {
    json!({
        "id": "66666666-6666-6666-6666-666666666666",
        "nameEnglish": "Test Team",
        "nameFrench": "Équipe test",
        "descriptionEnglish": "A team for testing",
        "descriptionFrench": "Une équipe de test",
        "retiredAt": null,
        "organization": {"id": "11111111-1111-1111-1111-111111111111", "nameEn": "Test Organization"},
        "organizationLevel": {"id": "22222222-2222-2222-2222-222222222222", "nameEn": "Test Tier"},
        "owner": {"id": "44444444-4444-4444-4444-444444444444", "givenName": "Jane", "familyName": "Doe", "email": "jane@example.com"},
        "occupiedRoles": [],
        "vacantRoles": [],
    })
}

fn sample_person() -> serde_json::Value {
    json!({
        "id": "88888888-8888-8888-8888-888888888888",
        "userEmail": "",
        "givenName": "Sam",
        "familyName": "Lee",
        "email": "sam.lee@example.com",
        "phone": "555-0100",
        "workAddress": "100 Main St",
        "city": "Ottawa",
        "province": "ON",
        "postalCode": "K1A0A1",
        "country": "Canada",
        "peoplesoftId": "PS-1",
        "orcidId": "",
        "retiredAt": null,
        "organization": {"id": "11111111-1111-1111-1111-111111111111", "nameEn": "Test Organization"},
        "capabilities": [],
        "activeRoles": [],
        "inactiveRoles": [],
        "findMatches": [],
        "affiliations": [],
        "publications": [],
    })
}

#[test]
fn team_form_renders_for_create_and_edit() {
    let tera = tera();
    for edit in [false, true] {
        let mut ctx = base_context("en", "operator");
        ctx.insert("edit", &edit);
        ctx.insert("team", &sample_team());
        ctx.insert("skill_domains", &domain_options());
        ctx.insert("org_tier_options", &parent_options());
        let html = tera.render("team/team_form.html", &ctx).unwrap();
        if edit {
            assert!(html.contains("/team/66666666-6666-6666-6666-666666666666/edit"));
            // tier can't change after creation
            assert!(html.contains("name=\"org_tier_id\" value=\"22222222-2222-2222-2222-222222222222\""));
        } else {
            assert!(html.contains("/team/new"));
            assert!(html.contains("name=\"org_tier_id\""));
        }
    }
}

#[test]
fn team_retire_page_renders() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("team", &sample_team());
    let html = tera.render("team/team_retire.html", &ctx).unwrap();
    assert!(html.contains("/team/66666666-6666-6666-6666-666666666666/retire"));
}

#[test]
fn org_chart_add_team_form_partial_renders() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("team", &sample_team());
    ctx.insert("skill_domains", &domain_options());
    let html = tera.render("org_chart/add_team_form.html", &ctx).unwrap();
    assert!(html.contains("hx-post=\"/en/team/new\""));
    assert!(html.contains("name=\"org_tier_id\" value=\"22222222-2222-2222-2222-222222222222\""));
}

#[test]
fn person_form_renders_for_create_and_edit() {
    let tera = tera();
    for edit in [false, true] {
        let mut ctx = base_context("en", "operator");
        ctx.insert("edit", &edit);
        ctx.insert("person", &sample_person());
        ctx.insert("organization_options", &json!([
            {"value": "11111111-1111-1111-1111-111111111111", "label": "Test Organization"}
        ]));
        let html = tera.render("person/person_form.html", &ctx).unwrap();
        if edit {
            assert!(html.contains("/person/88888888-8888-8888-8888-888888888888/edit"));
            // user account link can't change after creation
            assert!(!html.contains("name=\"user_email\""));
        } else {
            assert!(html.contains("/person/new"));
            assert!(html.contains("name=\"user_email\""));
        }
        // organization pre-selected
        assert!(html.contains("value=\"11111111-1111-1111-1111-111111111111\" selected"));
    }
}

#[test]
fn person_retire_page_renders() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("person", &sample_person());
    let html = tera.render("person/person_retire.html", &ctx).unwrap();
    assert!(html.contains("/person/88888888-8888-8888-8888-888888888888/retire"));
    assert!(html.contains("Sam Lee"));
}

fn sample_role_form() -> serde_json::Value {
    json!({
        "titleEnglish": "Analyst",
        "titleFrench": "Analyste",
        "effort": 1.0,
        "militaryOccupation": "CYBER",
        "rank": "CAPTAIN",
        "startDate": "2026-06-12",
        "personName": "",
        "teamId": "66666666-6666-6666-6666-666666666666",
        "orgTierId": "22222222-2222-2222-2222-222222222222",
        "organizationId": "11111111-1111-1111-1111-111111111111",
    })
}

fn sample_role_record() -> serde_json::Value {
    json!({
        "id": "77777777-7777-7777-7777-777777777777",
        "titleEnglish": "Analyst",
        "titleFrench": "Analyste",
        "active": "true",
        "militaryOccupation": "CYBER",
        "rank": "CAPTAIN",
        "effort": 1,
        "startDate": "2026-01-01",
        "endDate": "",
        "person": {"id": "88888888-8888-8888-8888-888888888888", "givenName": "Sam", "familyName": "Lee", "phone": "555", "email": "s@e.com"},
        "team": {
            "id": "66666666-6666-6666-6666-666666666666", "nameEnglish": "Test Team",
            "organizationLevel": {"nameEn": "Tier", "primaryDomain": "CYBER_SECURITY"},
            "owner": {"id": "44444444-4444-4444-4444-444444444444", "givenName": "Jane", "familyName": "Doe", "email": "j@e.com"},
        },
        "work": [],
        "requirements": [],
        "findMatches": [],
    })
}

fn role_enum_options() -> (serde_json::Value, serde_json::Value) {
    (
        json!([{"value": "CAPTAIN", "label": "Captain"}]),
        json!([{"value": "CYBER", "label": "Cyber"}]),
    )
}

#[test]
fn role_form_renders_full_page_and_partial() {
    let tera = tera();
    let (ranks, occupations) = role_enum_options();
    for template in ["role/role_form.html", "org_chart/add_role_form.html"] {
        let mut ctx = base_context("en", "operator");
        ctx.insert("role_form", &sample_role_form());
        ctx.insert("team", &sample_team());
        ctx.insert("ranks", &ranks);
        ctx.insert("military_occupations", &occupations);
        let html = tera.render(template, &ctx).unwrap();
        assert!(html.contains("name=\"team_id\" value=\"66666666-6666-6666-6666-666666666666\""));
        assert!(html.contains("value=\"CAPTAIN\" selected"));
        assert!(html.contains("name=\"person_name\""));
    }
}

#[test]
fn role_status_and_end_pages_render() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("role_record", &sample_role_record());
    let html = tera.render("role/role_status_form.html", &ctx).unwrap();
    // active checkbox pre-checked for an active role
    assert!(html.contains("name=\"active\""));
    assert!(html.contains("checked"));
    let html = tera.render("role/role_end.html", &ctx).unwrap();
    assert!(html.contains("/role/77777777-7777-7777-7777-777777777777/end"));
    assert!(html.contains("Sam Lee"));
}

#[test]
fn role_detail_shows_actions_for_operator_only() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("role_record", &sample_role_record());
    let html = tera.render("role/role.html", &ctx).unwrap();
    assert!(html.contains("/role/77777777-7777-7777-7777-777777777777/edit"));
    assert!(html.contains("/role/77777777-7777-7777-7777-777777777777/end"));

    let mut ctx = base_context("en", "user");
    ctx.insert("role_record", &sample_role_record());
    let html = tera.render("role/role.html", &ctx).unwrap();
    assert!(!html.contains("/edit"));
    assert!(!html.contains("/end"));
}

#[test]
fn org_chart_team_node_offers_add_role_for_operator() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("node", &sample_org_tier());
    let html = tera.render("org_chart/node.html", &ctx).unwrap();
    assert!(html.contains("role/new?team=66666666-6666-6666-6666-666666666666"));

    let mut ctx = base_context("en", "user");
    ctx.insert("node", &sample_org_tier());
    let html = tera.render("org_chart/node.html", &ctx).unwrap();
    assert!(!html.contains("role/new?team="));
}

#[test]
fn org_tier_assign_owner_page_renders() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("org_tier", &sample_org_tier());
    let html = tera.render("org_tier/assign_owner.html", &ctx).unwrap();
    assert!(html.contains("/org_tier/22222222-2222-2222-2222-222222222222/owner"));
    assert!(html.contains("name=\"person_name\""));
    assert!(html.contains("name=\"csrf_token\""));
}

#[test]
fn team_assign_owner_page_renders() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("team", &sample_team());
    let html = tera.render("team/assign_owner.html", &ctx).unwrap();
    assert!(html.contains("/team/66666666-6666-6666-6666-666666666666/owner"));
    assert!(html.contains("name=\"person_name\""));
}

#[test]
fn affiliation_form_renders() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("person", &sample_person());
    ctx.insert("affiliation", &json!({"organization": {"id": ""}, "affiliationRole": "", "endDate": ""}));
    ctx.insert("organization_options", &json!([
        {"value": "11111111-1111-1111-1111-111111111111", "label": "Test Organization"}
    ]));
    let html = tera.render("person/affiliation_form.html", &ctx).unwrap();
    assert!(html.contains("/person/88888888-8888-8888-8888-888888888888/affiliation/new"));
    assert!(html.contains("name=\"organization_id\""));
    assert!(html.contains("name=\"affiliation_role\""));
}

#[test]
fn person_page_affiliation_actions_operator_only() {
    let tera = tera();
    let mut person = sample_person();
    person["affiliations"] = json!([
        {"id": "aaaaaaaa-0000-0000-0000-000000000001", "organization": {"id": "11111111-1111-1111-1111-111111111111", "nameEn": "Partner Org"}, "affiliationRole": "Liaison"}
    ]);
    let mut ctx = base_context("en", "operator");
    ctx.insert("person", &person);
    let html = tera.render("person/person.html", &ctx).unwrap();
    assert!(html.contains("/person/88888888-8888-8888-8888-888888888888/affiliation/new"));
    assert!(html.contains("/affiliation/aaaaaaaa-0000-0000-0000-000000000001/end"));
    assert!(html.contains("Liaison"));

    let mut ctx = base_context("en", "user");
    ctx.insert("person", &person);
    let html = tera.render("person/person.html", &ctx).unwrap();
    assert!(!html.contains("/affiliation/new"));
    assert!(!html.contains("/end"));
}

fn sample_skill() -> serde_json::Value {
    json!({
        "id": "dddddddd-0000-0000-0000-000000000001",
        "nameEn": "Threat Analysis",
        "nameFr": "Analyse des menaces",
        "descriptionEn": "Assessing threats",
        "descriptionFr": "Évaluer les menaces",
        "domain": "CYBER_SECURITY",
        "capabilities": [
            {"id": "cccccccc-0000-0000-0000-000000000001", "selfIdentifiedLevel": "EXPERT", "validatedLevel": "EXPERIENCED",
             "person": {"id": "88888888-8888-8888-8888-888888888888", "givenName": "Sam", "familyName": "Lee"}}
        ],
    })
}

#[test]
fn skill_index_renders_with_operator_actions() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("skills", &json!([{"id": "dddddddd-0000-0000-0000-000000000001", "nameEn": "Threat Analysis", "domain": "CYBER_SECURITY", "retiredAt": null}]));
    let html = tera.render("skill/skill_index.html", &ctx).unwrap();
    assert!(html.contains("/en/skill/dddddddd-0000-0000-0000-000000000001"));
    assert!(html.contains("/en/skill/new"));

    let mut ctx = base_context("en", "user");
    ctx.insert("skills", &json!([]));
    let html = tera.render("skill/skill_index.html", &ctx).unwrap();
    assert!(!html.contains("/skill/new"));
}

#[test]
fn skill_detail_and_form_render() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("skill", &sample_skill());
    let html = tera.render("skill/skill.html", &ctx).unwrap();
    assert!(html.contains("Threat Analysis"));
    assert!(html.contains("/skill/dddddddd-0000-0000-0000-000000000001/edit"));
    assert!(html.contains("Sam Lee"));

    for edit in [false, true] {
        let mut ctx = base_context("en", "operator");
        ctx.insert("edit", &edit);
        ctx.insert("skill", &sample_skill());
        ctx.insert("skill_domains", &domain_options());
        let html = tera.render("skill/skill_form.html", &ctx).unwrap();
        if edit {
            assert!(html.contains("/skill/dddddddd-0000-0000-0000-000000000001/edit"));
        } else {
            assert!(html.contains("/skill/new"));
        }
    }
}

#[test]
fn capability_form_renders() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("person", &sample_person());
    ctx.insert("skill_options", &json!([{"value": "dddddddd-0000-0000-0000-000000000001", "label": "Threat Analysis"}]));
    ctx.insert("capability_levels", &json!([{"value": "EXPERT", "label": "Expert"}]));
    let html = tera.render("capability/capability_form.html", &ctx).unwrap();
    assert!(html.contains("/person/88888888-8888-8888-8888-888888888888/capability/new"));
    assert!(html.contains("name=\"skill_id\""));
    assert!(html.contains("name=\"self_identified_level\""));
}

#[test]
fn person_page_capability_actions_operator_only() {
    let tera = tera();
    let mut person = sample_person();
    person["capabilities"] = json!([
        {"id": "cccccccc-0000-0000-0000-000000000001", "nameEn": "Threat Analysis", "domain": "CYBER_SECURITY", "selfIdentifiedLevel": "EXPERT", "validatedLevel": "EXPERIENCED"}
    ]);
    let mut ctx = base_context("en", "operator");
    ctx.insert("person", &person);
    let html = tera.render("person/person.html", &ctx).unwrap();
    assert!(html.contains("/person/88888888-8888-8888-8888-888888888888/capability/new"));
    assert!(html.contains("/capability/cccccccc-0000-0000-0000-000000000001/retire"));

    let mut ctx = base_context("en", "user");
    ctx.insert("person", &person);
    let html = tera.render("person/person.html", &ctx).unwrap();
    assert!(!html.contains("/capability/new"));
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
