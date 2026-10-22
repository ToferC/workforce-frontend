// Form input contract: what a browser actually submits must deserialize, and
// the form macros must render the states the handlers rely on.

use fluent_templates::FluentLoader;
use serde_json::json;
use tera::{Context, Tera};

use frontend::handlers::{ContractEditForm, ContractForm, MyProfileForm, PersonForm, RoleForm};
use frontend::LOCALES;

/// An empty GCDS field is left out of the submission entirely, so every form
/// struct has to accept a post carrying only the fields that were filled.
#[test]
fn form_structs_accept_omitted_fields() {
    let contract: ContractForm = serde_urlencoded::from_str("csrf_token=t&reference_number=R-1").unwrap();
    assert_eq!(contract.description, "");
    let edit: ContractEditForm = serde_urlencoded::from_str("csrf_token=t&task_id=42&vendor=V").unwrap();
    assert_eq!((edit.task_id.as_str(), edit.contract.vendor.as_str()), ("42", "V"));
    let profile: MyProfileForm = serde_urlencoded::from_str("csrf_token=t&given_name=Sam").unwrap();
    assert_eq!(profile.phone, "");
    let _: PersonForm = serde_urlencoded::from_str("csrf_token=t").unwrap();
    let role: RoleForm = serde_urlencoded::from_str("csrf_token=t&effort=2.5").unwrap();
    assert_eq!(role.effort, 2.5);
}

fn render(body: &str, ctx: Context) -> String {
    let mut tera = Tera::new("templates/**/*").unwrap();
    tera.register_function("fluent", FluentLoader::new(&*LOCALES));
    tera.add_raw_template("probe.html", &format!("{{% import \"macros/forms.html\" as forms %}}{}", body)).unwrap();
    tera.render("probe.html", &ctx).unwrap()
}

#[test]
fn required_select_without_a_choice_shows_a_prompt_not_its_first_option() {
    let mut ctx = Context::new();
    ctx.insert("options", &json!([{"value": "COMBAT", "label": "Combat"}, {"value": "STRATEGY", "label": "Strategy"}]));
    let empty = render(r#"{{ forms::select(name="d", label="Domain", options=options) }}"#, ctx.clone());
    assert!(empty.contains(r#"<option value="" disabled selected>"#), "{}", empty);
    // A chosen value is set on the component and needs no prompt.
    let chosen = render(r#"{{ forms::select(name="d", label="Domain", options=options, selected="STRATEGY") }}"#, ctx.clone());
    assert!(chosen.contains(r#"value="STRATEGY""#) && !chosen.contains("disabled selected"));
    // Optional selects keep a choosable blank only when asked for one.
    let optional = render(r#"{{ forms::select(name="d", label="Domain", options=options, required=false) }}"#, ctx);
    assert!(!optional.contains("disabled"));
}

#[test]
fn inputs_render_step_and_field_errors() {
    let mut ctx = Context::new();
    ctx.insert("field_errors", &json!({"name_en": "Already in use", "end_date": "Too early"}));
    let html = render(
        r#"{{ forms::text_input(name="effort", label="Effort", type="number", step="any") }}
           {{ forms::text_input(name="name_en", label="Name", error=field_errors.name_en | default(value="")) }}
           {{ forms::date_input(name="end_date", label="End", error=field_errors.end_date | default(value="")) }}
           {{ forms::text_input(name="name_fr", label="Nom", error=field_errors.name_fr | default(value="")) }}"#,
        ctx,
    );
    assert!(html.contains(r#"step="any""#));
    assert!(html.contains(r#"error-message="Already in use""#));
    assert!(html.contains("Too early") && html.contains(r#"aria-invalid="true""#));
    assert_eq!(html.matches("error-message=").count(), 1, "fields without an error stay clean");
}

#[test]
fn filter_select_keeps_a_native_select_usable_without_js() {
    let mut ctx = Context::new();
    ctx.insert("options", &json!([{"value": "r1", "label": "Director General"}, {"value": "r2", "label": "Analyst"}]));
    let html = render(
        r#"{{ forms::filter_select(name="owner", label="Owner", options=options, filter_label="Filter", selected="r2", lang="fr") }}"#,
        ctx,
    );
    // The filter box ships hidden; forms.js reveals it.
    assert!(html.contains(r#"data-filter-for="owner""#) && html.contains(" hidden>"));
    assert!(html.contains(r#"<select class="form-select" id="owner" name="owner" required"#));
    assert!(html.contains(r#"<option value="r2" selected>Analyst</option>"#));
    assert!(html.contains("(obligatoire)"));
}
