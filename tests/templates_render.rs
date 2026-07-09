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
    // a retired org offers Restore instead of Retire
    assert!(html.contains("/organization/11111111-1111-1111-1111-111111111111/restore"));
}

#[test]
fn team_detail_uses_still_active_sentinel_for_restore() {
    let tera = tera();
    // active team (sentinel) -> Retire shown, no badge, no Restore
    let mut ctx = base_context("en", "operator");
    let mut team = sample_team();
    team["retiredAt"] = json!("Still Active");
    ctx.insert("team", &team);
    team_page_extras(&mut ctx);
    let html = tera.render("team/team.html", &ctx).unwrap();
    assert!(html.contains("/team/66666666-6666-6666-6666-666666666666/retire"));
    assert!(!html.contains("/restore"));

    // retired team (a date) -> Restore shown, no Retire
    let mut ctx = base_context("en", "operator");
    let mut team = sample_team();
    team["retiredAt"] = json!("2026-01-01");
    ctx.insert("team", &team);
    team_page_extras(&mut ctx);
    let html = tera.render("team/team.html", &ctx).unwrap();
    assert!(html.contains("/team/66666666-6666-6666-6666-666666666666/restore"));
    assert!(!html.contains("/team/66666666-6666-6666-6666-666666666666/retire"));
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
        "owner": {"person": {"id": "44444444-4444-4444-4444-444444444444", "givenName": "Jane", "familyName": "Doe", "email": "jane@example.com"}},
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
    ctx.insert("team_stats", &sample_team_stats());
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
        "headcount": 0,
        "totalEffort": 0,
        "organization": {"id": "11111111-1111-1111-1111-111111111111", "nameEn": "Test Organization"},
        "organizationLevel": {"id": "22222222-2222-2222-2222-222222222222", "nameEn": "Test Tier"},
        "owner": {"person": {"id": "44444444-4444-4444-4444-444444444444", "givenName": "Jane", "familyName": "Doe", "email": "jane@example.com"}},
        "occupiedRoles": [],
        "vacantRoles": [],
    })
}

// Per-team capability/capacity overlay stats as render_node_response builds
// them, keyed by team id (see org_chart.rs).
fn sample_team_stats() -> serde_json::Value {
    json!({
        "66666666-6666-6666-6666-666666666666": {
            "headcount": 1,
            "vacant": 1,
            "effort": 3,
            "top_domains": [{"label": "Cyber", "group": "digital"}],
            "capacity_class": "success",
        }
    })
}

// The delivery context team_by_id inserts alongside the team itself.
fn team_page_extras(ctx: &mut Context) {
    ctx.insert("products", &json!([]));
    ctx.insert("tasks", &json!([]));
    ctx.insert("active_work", &json!([]));
    ctx.insert("work_count", &0);
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
        "personnelType": "CIVILIAN",
        "retiredAt": null,
        "organization": {"id": "11111111-1111-1111-1111-111111111111", "nameEn": "Test Organization"},
        "capabilities": [],
        "languageData": [],
        "activeRoles": [],
        "inactiveRoles": [],
        "roleAssignments": [],
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
        ctx.insert("personnel_types", &json!([
            {"value": "CIVILIAN", "label": "Civilian"}, {"value": "MILITARY", "label": "Military"}
        ]));
        let html = tera.render("person/person_form.html", &ctx).unwrap();
        if edit {
            assert!(html.contains("/person/88888888-8888-8888-8888-888888888888/edit"));
        } else {
            assert!(html.contains("/person/new"));
        }
        // account linking moved to the admin invite / grant-access flow
        assert!(!html.contains("name=\"user_email\""));
        assert!(html.contains("name=\"personnel_type\""));
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
        "occupationalGroup": "",
        "occupationalLevel": "",
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
        "assignments": [],
    })
}

// A scored candidate as the fuzzyMatches resolver serializes it.
fn sample_match_score(given: &str, family: &str, score: f64, met: i64, total: i64, full: bool) -> serde_json::Value {
    json!({
        "matchScore": score,
        "coverage": met as f64 / total as f64,
        "requirementsMet": met,
        "requirementsTotal": total,
        "totalGap": total - met,
        "requirementGaps": [
            {"skillName": "Threat Analysis", "requiredLevel": "EXPERT", "actualLevel": "EXPERT", "gap": 0, "met": true},
            {"skillName": "Incident Response", "requiredLevel": "EXPERT", "actualLevel": if full { "EXPERT" } else { "EXPERIENCED" }, "gap": if full { 0 } else { 1 }, "met": full},
        ],
        "person": {
            "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa", "givenName": given, "familyName": family,
            "phone": "555", "email": "c@e.com",
            "activeRoles": [{"id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb", "titleEnglish": "Analyst", "militaryOccupation": "CYBER", "rank": "CAPTAIN"}],
            "capabilities": [],
        },
    })
}

fn role_enum_options() -> (serde_json::Value, serde_json::Value) {
    (
        json!([{"value": "CAPTAIN", "label": "Captain"}]),
        json!([{"value": "CYBER", "label": "Cyber"}]),
    )
}

fn occupational_group_options() -> serde_json::Value {
    json!([{"value": "COMPUTER_SYSTEMS", "label": "Computer Systems"}])
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
        ctx.insert("occupational_groups", &occupational_group_options());
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
fn role_detail_shows_assignment_history() {
    let tera = tera();
    let mut role = sample_role_record();
    role["assignments"] = json!([
        {"id": "a0000000-0000-0000-0000-000000000001", "startDate": "2026-01-01", "endDate": "Current", "isCurrent": true,
         "person": {"id": "88888888-8888-8888-8888-888888888888", "givenName": "Sam", "familyName": "Lee"}},
        {"id": "a0000000-0000-0000-0000-000000000002", "startDate": "2024-01-01", "endDate": "2025-12-31", "isCurrent": false,
         "person": {"id": "99999999-9999-9999-9999-999999999999", "givenName": "Pat", "familyName": "Kim"}},
    ]);
    let mut ctx = base_context("en", "operator");
    ctx.insert("role_record", &role);
    let html = tera.render("role/role.html", &ctx).unwrap();
    assert!(html.contains("Assignment History"));
    // both occupants and the current badge appear
    assert!(html.contains("Sam Lee"));
    assert!(html.contains("Pat Kim"));
    assert!(html.contains("Current"));
    assert!(html.contains("/person/99999999-9999-9999-9999-999999999999"));
}

#[test]
fn role_detail_vacant_renders_fuzzy_match_panel() {
    let tera = tera();
    let mut role = sample_role_record();
    role["person"] = json!(null);
    role["requirements"] = json!([
        {"id": "r0000000-0000-0000-0000-000000000001", "nameEn": "Threat Analysis", "domain": "CYBER_SECURITY", "requiredLevel": "EXPERT"},
    ]);
    // An external candidate carries their manager's contact for the offer flow
    let mut external = sample_match_score("Robin", "Sage", 1.0, 1, 1, true);
    external["manager"] = json!({
        "ownerRoleId": "cccccccc-cccc-cccc-cccc-cccccccccccc",
        "ownerRoleTitle": "Section Head",
        "teamName": "Other Team",
        "name": "Morgan Hale",
        "email": "morgan@example.com",
        "phone": "555-0199",
    });

    let mut ctx = base_context("en", "operator");
    ctx.insert("role_record", &role);
    ctx.insert("match_role_id", "77777777-7777-7777-7777-777777777777");
    ctx.insert("min_coverage_pct", &50);
    ctx.insert("max_gap_per_req", &1);
    ctx.insert("match_managed_full", &json!([sample_match_score("Alex", "Roy", 1.0, 1, 1, true)]));
    ctx.insert("match_managed_partial", &json!([sample_match_score("Jamie", "Fox", 0.65, 3, 4, false)]));
    ctx.insert("match_external_full", &json!([external]));
    ctx.insert("match_external_partial", &json!([]));
    let html = tera.render("role/role.html", &ctx).unwrap();
    // Tuning sliders present and wired to the HTMX endpoint
    assert!(html.contains("name=\"min_coverage\""));
    assert!(html.contains("name=\"max_gap_per_req\""));
    assert!(html.contains("/role/77777777-7777-7777-7777-777777777777/matches"));
    // Managed and external sections render their candidates and scores
    assert!(html.contains("In your area"));
    assert!(html.contains("Elsewhere in the organization"));
    assert!(html.contains("Full matches"));
    assert!(html.contains("Close matches"));
    assert!(html.contains("Alex Roy"));
    assert!(html.contains("Jamie Fox"));
    assert!(html.contains("65% match"));
    // Shortfall breakdown surfaces the gap
    assert!(html.contains("Incident Response"));
    // External candidate shows manager contact and the offer action
    assert!(html.contains("Robin Sage"));
    assert!(html.contains("Morgan Hale"));
    assert!(html.contains("/role/77777777-7777-7777-7777-777777777777/offer"));
}

#[test]
fn role_matches_partial_renders_standalone() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("match_role_id", "77777777-7777-7777-7777-777777777777");
    ctx.insert("min_coverage_pct", &30);
    ctx.insert("max_gap_per_req", &2);
    ctx.insert("match_managed_full", &json!([]));
    ctx.insert("match_managed_partial", &json!([sample_match_score("Jamie", "Fox", 0.5, 2, 4, false)]));
    ctx.insert("match_external_full", &json!([]));
    ctx.insert("match_external_partial", &json!([]));
    let html = tera.render("role/_matches.html", &ctx).unwrap();
    assert!(html.contains("No one in your area meets every requirement."));
    assert!(html.contains("Jamie Fox"));
    // Threshold caption reflects the slider values
    assert!(html.contains("30%"));
}

#[test]
fn person_page_shows_past_roles_from_assignments() {
    let tera = tera();
    let mut person = sample_person();
    // One current (excluded) and one closed (shown as a past role)
    person["roleAssignments"] = json!([
        {"id": "a0000000-0000-0000-0000-000000000003", "startDate": "2026-01-01", "endDate": "Current", "isCurrent": true,
         "role": {"id": "77777777-7777-7777-7777-777777777777", "titleEnglish": "Analyst", "militaryOccupation": "CYBER", "rank": "CAPTAIN", "team": {"id": "66666666-6666-6666-6666-666666666666", "nameEnglish": "Test Team"}}},
        {"id": "a0000000-0000-0000-0000-000000000004", "startDate": "2023-01-01", "endDate": "2025-12-31", "isCurrent": false,
         "role": {"id": "55555555-5555-5555-5555-555555555555", "titleEnglish": "Junior Analyst", "militaryOccupation": "CYBER", "rank": "LIEUTENANT", "team": {"id": "66666666-6666-6666-6666-666666666666", "nameEnglish": "Old Team"}}},
    ]);
    let mut ctx = base_context("en", "operator");
    ctx.insert("person", &person);
    let html = tera.render("person/person.html", &ctx).unwrap();
    // the closed tenure shows as a past role with its team and dates
    assert!(html.contains("Junior Analyst"));
    assert!(html.contains("Old Team"));
    assert!(html.contains("2025-12-31"));
    // the current tenure is not listed under Past Roles
    assert!(!html.contains("/role/77777777-7777-7777-7777-777777777777"));
}

#[test]
fn org_chart_team_node_offers_add_role_for_operator() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("node", &sample_org_tier());
    ctx.insert("team_stats", &sample_team_stats());
    let html = tera.render("org_chart/node.html", &ctx).unwrap();
    assert!(html.contains("role/new?team=66666666-6666-6666-6666-666666666666"));

    let mut ctx = base_context("en", "user");
    ctx.insert("node", &sample_org_tier());
    ctx.insert("team_stats", &sample_team_stats());
    let html = tera.render("org_chart/node.html", &ctx).unwrap();
    assert!(!html.contains("role/new?team="));
}

#[test]
fn org_tier_assign_owner_page_renders() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("org_tier", &sample_org_tier());
    ctx.insert("role_options", &json!([{"value": "77777777-7777-7777-7777-777777777777", "label": "Analyst — Sam Lee"}]));
    let html = tera.render("org_tier/assign_owner.html", &ctx).unwrap();
    assert!(html.contains("/org_tier/22222222-2222-2222-2222-222222222222/owner"));
    assert!(html.contains("name=\"owner_role_id\""));
    assert!(html.contains("name=\"csrf_token\""));
}

#[test]
fn team_assign_owner_page_renders() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("team", &sample_team());
    ctx.insert("role_options", &json!([{"value": "77777777-7777-7777-7777-777777777777", "label": "Analyst — Sam Lee"}]));
    let html = tera.render("team/assign_owner.html", &ctx).unwrap();
    assert!(html.contains("/team/66666666-6666-6666-6666-666666666666/owner"));
    assert!(html.contains("name=\"owner_role_id\""));
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

// Domain-grouped skill options as skill_picker_data builds them for the
// two-step skill picker (skill/skill_picker.html).
fn skill_group_options() -> serde_json::Value {
    json!([{
        "value": "CYBER_SECURITY",
        "label": "Cyber Security",
        "skills": [{"value": "dddddddd-0000-0000-0000-000000000001", "label": "Threat Analysis"}],
    }])
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
    ctx.insert("q", "");
    ctx.insert("show_retired", &false);
    let html = tera.render("skill/skill_index.html", &ctx).unwrap();
    assert!(html.contains("/en/skill/dddddddd-0000-0000-0000-000000000001"));
    assert!(html.contains("/en/skill/new"));

    let mut ctx = base_context("en", "user");
    ctx.insert("skills", &json!([]));
    ctx.insert("q", "");
    ctx.insert("show_retired", &false);
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
    ctx.insert("skill_domains", &domain_options());
    ctx.insert("skill_groups", &skill_group_options());
    ctx.insert("capability_levels", &json!([{"value": "EXPERT", "label": "Expert"}]));
    let html = tera.render("capability/capability_form.html", &ctx).unwrap();
    assert!(html.contains("/person/88888888-8888-8888-8888-888888888888/capability/new"));
    // Two-step skill picker: domain select + grouped skill select
    assert!(html.contains("name=\"domain\""));
    assert!(html.contains("name=\"skill_id\""));
    assert!(html.contains("Threat Analysis"));
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
fn requirement_form_renders() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("role_id", "77777777-7777-7777-7777-777777777777");
    ctx.insert("skill_domains", &domain_options());
    ctx.insert("skill_groups", &skill_group_options());
    ctx.insert("capability_levels", &json!([{"value": "EXPERT", "label": "Expert"}]));
    let html = tera.render("role/requirement_form.html", &ctx).unwrap();
    assert!(html.contains("/role/77777777-7777-7777-7777-777777777777/requirement/new"));
    assert!(html.contains("name=\"domain\""));
    assert!(html.contains("name=\"skill_id\""));
    assert!(html.contains("name=\"required_level\""));
}

#[test]
fn role_page_requirement_actions_operator_only() {
    let tera = tera();
    let mut role = sample_role_record();
    role["requirements"] = json!([{"id": "eeee0000-0000-0000-0000-000000000001", "nameEn": "Threat Analysis", "domain": "CYBER_SECURITY", "requiredLevel": "EXPERT"}]);
    let mut ctx = base_context("en", "operator");
    ctx.insert("role_record", &role);
    let html = tera.render("role/role.html", &ctx).unwrap();
    assert!(html.contains("/role/77777777-7777-7777-7777-777777777777/requirement/new"));
    assert!(html.contains("/requirement/eeee0000-0000-0000-0000-000000000001/retire"));

    let mut ctx = base_context("en", "user");
    ctx.insert("role_record", &role);
    let html = tera.render("role/role.html", &ctx).unwrap();
    assert!(!html.contains("/requirement/new"));
}

#[test]
fn validation_form_renders() {
    let tera = tera();
    let mut ctx = base_context("en", "admin");
    ctx.insert("person_id", "88888888-8888-8888-8888-888888888888");
    ctx.insert("capability_id", "cccccccc-0000-0000-0000-000000000001");
    ctx.insert("capability_levels", &json!([{"value": "EXPERT", "label": "Expert"}]));
    ctx.insert("validator_display", "Admin Tester");
    let html = tera.render("capability/validation_form.html", &ctx).unwrap();
    assert!(html.contains("/person/88888888-8888-8888-8888-888888888888/capability/cccccccc-0000-0000-0000-000000000001/validate"));
    // The validator is the logged-in admin, shown (not typed) on the form
    assert!(html.contains("Admin Tester"));
    assert!(html.contains("name=\"validated_level\""));
}

#[test]
fn language_form_renders() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("person", &sample_person());
    ctx.insert("language_names", &json!([{"value": "ENGLISH", "label": "English"}]));
    ctx.insert("language_levels", &json!([{"value": "C", "label": "C"}]));
    let html = tera.render("person/language_form.html", &ctx).unwrap();
    assert!(html.contains("/person/88888888-8888-8888-8888-888888888888/language/new"));
    assert!(html.contains("name=\"language_name\""));
    assert!(html.contains("name=\"reading\""));
}

#[test]
fn person_page_shows_languages_and_validate_for_admin() {
    let tera = tera();
    let mut person = sample_person();
    person["languageData"] = json!([{"id": "ffff0000-0000-0000-0000-000000000001", "languageName": "FRENCH", "reading": "C", "writing": "B", "speaking": "C"}]);
    person["capabilities"] = json!([{"id": "cccccccc-0000-0000-0000-000000000001", "nameEn": "Threat Analysis", "domain": "CYBER_SECURITY", "selfIdentifiedLevel": "EXPERT", "validatedLevel": null}]);
    // admin sees validate
    let mut ctx = base_context("en", "admin");
    ctx.insert("person", &person);
    let html = tera.render("person/person.html", &ctx).unwrap();
    assert!(html.contains("/person/88888888-8888-8888-8888-888888888888/language/new"));
    assert!(html.contains("FRENCH"));
    assert!(html.contains("/capability/cccccccc-0000-0000-0000-000000000001/validate"));
    // operator does NOT see validate (admin-only) but sees retire
    let mut ctx = base_context("en", "operator");
    ctx.insert("person", &person);
    let html = tera.render("person/person.html", &ctx).unwrap();
    assert!(!html.contains("/validate"));
    assert!(html.contains("/capability/cccccccc-0000-0000-0000-000000000001/retire"));
}

fn status_options() -> serde_json::Value {
    json!([{"value": "PLANNING", "label": "Planning"}, {"value": "IN_PROGRESS", "label": "In Progress"}])
}

fn priority_options() -> serde_json::Value {
    json!([{"value": "LOW", "label": "Low"}, {"value": "MEDIUM", "label": "Medium"}, {"value": "HIGH", "label": "High"}])
}

#[test]
fn task_index_and_form_render() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("tasks", &json!([{"id": "a0000000-0000-0000-0000-000000000001", "title": "Stand up cyber cell", "domain": "CYBER_SECURITY", "taskStatus": "PLANNING"}]));
    let html = tera.render("task/task_index.html", &ctx).unwrap();
    assert!(html.contains("/en/task/a0000000-0000-0000-0000-000000000001"));

    let task = json!({"id": "a0000000-0000-0000-0000-000000000001", "title": "Stand up cyber cell", "domain": "CYBER_SECURITY",
        "intendedOutcome": "Cell operational", "finalOutcome": "", "approvalTier": 2, "url": "",
        "startDatestamp": "2026-01-01", "targetCompletionDate": "2026-06-01", "taskStatus": "PLANNING", "completedDate": "",
        "priority": "MEDIUM"});
    for (edit, action) in [(false, "/role/77777777-7777-7777-7777-777777777777/task/new"), (true, "/task/a0000000-0000-0000-0000-000000000001/edit")] {
        let mut ctx = base_context("en", "operator");
        ctx.insert("edit", &edit);
        ctx.insert("role_id", "77777777-7777-7777-7777-777777777777");
        ctx.insert("task", &task);
        ctx.insert("skill_domains", &domain_options());
        ctx.insert("work_statuses", &status_options());
        ctx.insert("priorities", &priority_options());
        ctx.insert("product_options", &json!([{"value": "d0000000-0000-0000-0000-000000000001", "label": "A product"}]));
        let html = tera.render("task/task_form.html", &ctx).unwrap();
        assert!(html.contains(action));
        assert!(html.contains("name=\"task_status\""));
        assert!(html.contains("name=\"priority\""));
        assert!(html.contains("name=\"product_id\""));
    }
}

#[test]
fn work_form_renders() {
    let tera = tera();
    let work = json!({"id": "b0000000-0000-0000-0000-000000000001", "workDescription": "Draft plan", "url": "",
        "domain": "CYBER_SECURITY", "capabilityLevel": "EXPERT", "effort": 3, "workStatus": "PLANNING",
        "priority": "MEDIUM", "dueDate": ""});
    // create: task select present
    let mut ctx = base_context("en", "operator");
    ctx.insert("edit", &false);
    ctx.insert("vacant", &false);
    ctx.insert("role_id", "77777777-7777-7777-7777-777777777777");
    ctx.insert("skill_id", "");
    ctx.insert("domain", "");
    ctx.insert("work", &work);
    ctx.insert("task_options", &json!([{"value": "a0000000-0000-0000-0000-000000000001", "label": "A task"}]));
    ctx.insert("skill_options", &json!([{"value": "dddddddd-0000-0000-0000-000000000001", "label": "Threat Analysis"}]));
    ctx.insert("skill_domains", &domain_options());
    ctx.insert("capability_levels", &json!([{"value": "EXPERT", "label": "Expert"}]));
    ctx.insert("work_statuses", &status_options());
    ctx.insert("priorities", &priority_options());
    let html = tera.render("work/work_form.html", &ctx).unwrap();
    assert!(html.contains("/role/77777777-7777-7777-7777-777777777777/work/new"));
    assert!(html.contains("name=\"task_id\""));
    assert!(html.contains("name=\"priority\""));
    assert!(html.contains("name=\"due_date\""));
    // edit: no task select; blocked-context fields offered
    let mut ctx = base_context("en", "operator");
    ctx.insert("edit", &true);
    ctx.insert("skill_id", "dddddddd-0000-0000-0000-000000000001");
    ctx.insert("domain", "CYBER_SECURITY");
    ctx.insert("work", &work);
    ctx.insert("skill_options", &json!([{"value": "dddddddd-0000-0000-0000-000000000001", "label": "Threat Analysis"}]));
    ctx.insert("blocked_role_options", &json!([{"value": "77777777-7777-7777-7777-777777777777", "label": "Analyst"}]));
    ctx.insert("skill_domains", &domain_options());
    ctx.insert("capability_levels", &json!([{"value": "EXPERT", "label": "Expert"}]));
    ctx.insert("work_statuses", &status_options());
    ctx.insert("priorities", &priority_options());
    let html = tera.render("work/work_form.html", &ctx).unwrap();
    assert!(html.contains("/work/b0000000-0000-0000-0000-000000000001/edit"));
    assert!(!html.contains("name=\"task_id\""));
    assert!(html.contains("name=\"blocked_reason\""));
    assert!(html.contains("name=\"blocked_on_role_id\""));
}

#[test]
fn publication_index_and_form_render() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("publications", &json!([{"id": "c0000000-0000-0000-0000-000000000001", "title": "Threat report", "publicationStatus": "DRAFT"}]));
    let html = tera.render("publication/publication_index.html", &ctx).unwrap();
    assert!(html.contains("/en/publication/c0000000-0000-0000-0000-000000000001"));
    assert!(html.contains("/en/publication/new"));

    let pubn = json!({"id": "c0000000-0000-0000-0000-000000000001", "title": "Threat report", "subjectText": "Threats",
        "publicationStatus": "DRAFT", "urlString": "", "publishingId": "", "publishedDatestamp": "",
        "publishingOrganization": {"id": "11111111-1111-1111-1111-111111111111"}});
    // create: org select + lead author present
    let mut ctx = base_context("en", "operator");
    ctx.insert("edit", &false);
    ctx.insert("publication", &pubn);
    ctx.insert("organization_options", &json!([{"value": "11111111-1111-1111-1111-111111111111", "label": "Test Org"}]));
    ctx.insert("publication_statuses", &status_options());
    let html = tera.render("publication/publication_form.html", &ctx).unwrap();
    assert!(html.contains("/publication/new"));
    assert!(html.contains("name=\"lead_author_name\""));
    // edit: no org/author
    let mut ctx = base_context("en", "operator");
    ctx.insert("edit", &true);
    ctx.insert("publication", &pubn);
    ctx.insert("publication_statuses", &status_options());
    let html = tera.render("publication/publication_form.html", &ctx).unwrap();
    assert!(html.contains("/publication/c0000000-0000-0000-0000-000000000001/edit"));
    assert!(!html.contains("name=\"lead_author_name\""));
}

#[test]
fn team_index_renders_with_retired_toggle() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("teams", &json!([
        {"id": "66666666-6666-6666-6666-666666666666", "nameEnglish": "Active Team", "nameFrench": "x", "retiredAt": "Still Active", "organization": {"id": "1", "nameEn": "Org"}, "organizationLevel": {"id": "2", "nameEn": "Tier"}},
        {"id": "66666666-6666-6666-6666-666666666667", "nameEnglish": "Old Team", "nameFrench": "y", "retiredAt": "2026-01-01", "organization": {"id": "1", "nameEn": "Org"}, "organizationLevel": {"id": "2", "nameEn": "Tier"}}
    ]));
    ctx.insert("show_retired", &false);
    ctx.insert("q", "");
    ctx.insert("total", &2);
    ctx.insert("page", &1);
    ctx.insert("total_pages", &1);
    ctx.insert("has_prev", &false);
    ctx.insert("has_next", &false);
    let html = tera.render("team/team_index.html", &ctx).unwrap();
    assert!(html.contains("/teams?retired=1"));      // "show retired" link when hidden
    assert!(html.contains("Active Team"));
    // the retired one carries the badge
    assert!(html.contains("/team/66666666-6666-6666-6666-666666666667"));
    let badges = html.matches("badge bg-warning").count();
    assert_eq!(badges, 1, "only the retired team should be badged");

    ctx.insert("show_retired", &true);
    let html = tera.render("team/team_index.html", &ctx).unwrap();
    assert!(html.contains("/teams\""));               // "hide retired" link back to plain
}

#[test]
fn role_index_renders_vacant_and_occupied() {
    let tera = tera();
    let mut ctx = base_context("en", "user");
    ctx.insert("roles", &json!([
        {"id": "77777777-7777-7777-7777-777777777777", "titleEnglish": "Analyst", "titleFrench": "x", "militaryOccupation": "CYBER", "rank": "CAPTAIN", "person": {"id": "8", "givenName": "Sam", "familyName": "Lee"}, "team": {"id": "6", "nameEnglish": "Team"}},
        {"id": "77777777-7777-7777-7777-777777777778", "titleEnglish": "Advisor", "titleFrench": "y", "militaryOccupation": null, "rank": null, "person": null, "team": {"id": "6", "nameEnglish": "Team"}}
    ]));
    ctx.insert("q", "");
    ctx.insert("total", &2);
    ctx.insert("truncated", &false);
    ctx.insert("organizations", &json!([{"id": "1", "nameEn": "Alpha Org"}, {"id": "2", "nameEn": "Beta Org"}]));
    ctx.insert("selected_org", "2");
    ctx.insert("selected_status", "vacant");
    let html = tera.render("role/role_index.html", &ctx).unwrap();
    assert!(html.contains("Sam Lee"));
    assert!(html.contains("/role/77777777-7777-7777-7777-777777777778"));
    assert!(html.contains("badge bg-danger"));  // vacant badge for the unassigned role
    // Filter controls render with the org list and persist the selections
    assert!(html.contains("name=\"org\""));
    assert!(html.contains("name=\"status\""));
    assert!(html.contains("Alpha Org"));
    assert!(html.contains("value=\"2\" selected"));            // selected org persisted
    assert!(html.contains("value=\"vacant\" selected"));       // selected status persisted
}

#[test]
fn person_index_renders_with_retired_toggle() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("people", &json!([
        {"id": "88888888-8888-8888-8888-888888888888", "givenName": "Sam", "familyName": "Lee", "retiredAt": null, "organization": {"id": "1", "nameEn": "Org"}}
    ]));
    ctx.insert("show_retired", &false);
    ctx.insert("q", "");
    ctx.insert("total", &1);
    ctx.insert("page", &1);
    ctx.insert("total_pages", &1);
    ctx.insert("has_prev", &false);
    ctx.insert("has_next", &false);
    ctx.insert("organizations", &json!([{"id": "1", "nameEn": "Alpha Org"}]));
    ctx.insert("selected_org", "1");
    ctx.insert("selected_status", "available");
    let html = tera.render("person/person_index.html", &ctx).unwrap();
    assert!(html.contains("/people?retired=1"));
    assert!(html.contains("Sam Lee"));
    assert!(html.contains("/person/new"));  // operator sees New Person
    // Org + availability filters render and persist the active selections
    assert!(html.contains("name=\"org\""));
    assert!(html.contains("name=\"status\""));
    assert!(html.contains("Alpha Org"));
    assert!(html.contains("value=\"1\" selected"));            // selected org persisted
    assert!(html.contains("value=\"available\" selected"));    // availability persisted
}

#[test]
fn person_list_partial_renders_pagination() {
    let tera = tera();
    let mut ctx = base_context("en", "operator");
    ctx.insert("people", &json!([{"id": "88888888-8888-8888-8888-888888888888", "givenName": "Sam", "familyName": "Lee", "retiredAt": null, "organization": {"id": "1", "nameEn": "Org"}}]));
    ctx.insert("total", &250);
    ctx.insert("page", &1);
    ctx.insert("total_pages", &3);
    ctx.insert("has_prev", &false);
    ctx.insert("has_next", &true);
    ctx.insert("q", "");
    ctx.insert("show_retired", &false);
    ctx.insert("selected_org", "");
    ctx.insert("selected_status", "");
    // partial renders standalone (the HTMX swap target) with page controls
    let html = tera.render("person/person_list.html", &ctx).unwrap();
    assert!(html.contains("id=\"person-list\""));
    assert!(html.contains("250"));       // total shown in the page indicator
    assert!(html.contains("page=2"));    // next-page link
    assert!(!html.contains("page=0"));   // no previous link on page 1
}

#[test]
fn analytics_templates_render_in_both_languages() {
    let tera = tera();
    let chart = "{\"series\":[]}";
    for lang in ["en", "fr"] {
        // Dashboard shell (no data context — sections lazy-load)
        let ctx = base_context(lang, "analyst");
        let html = tera.render("analytics/analytics.html", &ctx).unwrap();
        assert!(html.contains("/analytics/coverage"));
        assert!(html.contains("/analytics/consistency"));

        // Section partials
        let mut ctx = base_context(lang, "analyst");
        ctx.insert("total_work", &12);
        ctx.insert("vacant_work", &2);
        ctx.insert("work_status_counts", &json!([{"status": "IN_PROGRESS", "count": 8}]));
        ctx.insert("work_by_domain", &json!([{"domain": "CYBER_SECURITY", "effort": 20}]));
        tera.render("analytics/_section_work.html", &ctx).unwrap();

        let mut ctx = base_context(lang, "analyst");
        ctx.insert("total_people", &40);
        ctx.insert("available_count", &5);
        ctx.insert("team_capacity", &json!([{"team": "Test Team", "effort": 30}]));
        ctx.insert("over_allocated", &json!([{"id": "88888888-8888-8888-8888-888888888888", "name": "Sam Lee", "team": "Test Team", "effort": 9}]));
        tera.render("analytics/_section_capacity.html", &ctx).unwrap();

        let mut ctx = base_context(lang, "analyst");
        ctx.insert("vacant_roles_count", &1);
        ctx.insert("vacant_roles", &json!([{"id": "77777777-7777-7777-7777-777777777777", "title": "Analyst", "team": "Test Team"}]));
        tera.render("analytics/_section_vacancies.html", &ctx).unwrap();

        let mut ctx = base_context(lang, "analyst");
        ctx.insert("domain_gaps", &json!([{
            "domain": "CYBER_SECURITY", "total_required": 4, "total_available": 2, "net_gap": 2,
            "levels": [{"level": "EXPERT", "required": 2, "available": 1, "gap": 1}],
        }]));
        tera.render("analytics/_section_gaps.html", &ctx).unwrap();

        // Coverage
        let mut ctx = base_context(lang, "analyst");
        ctx.insert("summary", &json!({"total_teams": 3, "active_domains": 2, "max_depth": 9}));
        ctx.insert("chart_height", "300px");
        ctx.insert("chart_option", chart);
        ctx.insert("domain_totals", &json!([{"key": "CYBER_SECURITY", "total": 9}]));
        ctx.insert("domain_labels", &json!(["Cyber"]));
        ctx.insert("table_rows", &json!([{
            "team_id": "66666666-6666-6666-6666-666666666666", "team": "Test Team",
            "cells": [{"opacity": 0.5, "depth": 9}],
        }]));
        tera.render("analytics/coverage.html", &ctx).unwrap();

        // Delivery
        let mut ctx = base_context(lang, "analyst");
        ctx.insert("summary", &json!({"total_products": 1, "total_tasks": 2, "total_work": 3, "total_effort": 9, "rendered_products": 1}));
        ctx.insert("status_legend", &json!([{"color": "#0d6efd", "status": "IN_PROGRESS"}]));
        ctx.insert("chart_option", chart);
        ctx.insert("product_rows", &json!([{
            "id": "d0000000-0000-0000-0000-000000000001", "name": "A product",
            "domain": "CYBER_SECURITY", "task_count": 2, "work_count": 3, "effort": 9,
        }]));
        tera.render("analytics/delivery.html", &ctx).unwrap();

        // Growth
        let mut ctx = base_context(lang, "analyst");
        ctx.insert("summary", &json!({"total_domains": 2, "latest_total": 40}));
        ctx.insert("chart_option", chart);
        tera.render("analytics/growth.html", &ctx).unwrap();

        // Mobility
        let mut ctx = base_context(lang, "analyst");
        ctx.insert("summary", &json!({"total_moves": 4, "promotions": 1, "laterals": 2, "inflows": 1, "outflows": 0, "org_tiers_involved": 3}));
        ctx.insert("has_moves", &true);
        ctx.insert("chart_option", chart);
        ctx.insert("table_rows", &json!([{"from": "Tier A", "to": "Tier B", "count": 2}]));
        tera.render("analytics/mobility.html", &ctx).unwrap();

        // Supply vs demand
        let mut ctx = base_context(lang, "analyst");
        ctx.insert("summary", &json!({"total_domains": 1, "surplus_count": 1, "deficit_count": 0}));
        ctx.insert("domain_charts", &json!([{
            "domain_key": "CYBER_SECURITY", "domain": "Cyber Security", "has_surplus": true,
            "gap": 3, "chart_option": chart, "latest_supply": 10, "latest_demand": 7,
        }]));
        tera.render("analytics/supply_demand.html", &ctx).unwrap();
    }

    // Spot-check the French actually comes through, not just that it renders
    let ctx = base_context("fr", "analyst");
    let html = tera.render("analytics/analytics.html", &ctx).unwrap();
    assert!(html.contains("Analytique de l") && html.contains("effectif"));
    assert!(html.contains("Mobilité des talents"));
}

#[test]
fn nav_offers_analytics_to_analysts_and_above_only() {
    let tera = tera();
    // Analytics handlers require MinimumRole::Analyst; the menu should match.
    for (role, expected) in [("user", false), ("analyst", true), ("operator", true), ("admin", true)] {
        let mut ctx = base_context("en", role);
        ctx.insert("organizations", &json!([]));
        let html = tera.render("index.html", &ctx).unwrap();
        assert_eq!(
            html.contains("/en/analytics"), expected,
            "role {} should{} see the analytics menu", role, if expected { "" } else { " not" }
        );
    }
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
