// Every API enum value rendered through labels::enum_label / frontend::enum_label
// must have a short label in both languages, so nothing falls back to raw
// ALL_CAPS text. Values are read from schema.graphql, so a new enum value in
// the API fails this test until its labels are added.

use std::collections::HashSet;

/// Label kind → the schema enums whose values it covers.
const KINDS: &[(&str, &[&str])] = &[
    ("domain", &["SkillDomain"]),
    ("level", &["CapabilityLevel"]),
    ("status", &["WorkStatus", "PublicationStatus"]),
    ("priority", &["Priority"]),
    ("personnel", &["PersonnelType"]),
];

fn enum_values(schema: &str, name: &str) -> Vec<String> {
    let start = schema.find(&format!("enum {} {{", name)).unwrap_or_else(|| panic!("enum {} not in schema", name));
    let body = &schema[start..];
    let body = &body[body.find('{').unwrap() + 1..body.find('}').unwrap()];
    body.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('"')).map(String::from).collect()
}

fn ftl_keys(lang: &str) -> HashSet<String> {
    std::fs::read_to_string(format!("i18n/{}/workforce.ftl", lang)).unwrap()
        .lines()
        .filter_map(|l| l.split_once(" = ").map(|(k, _)| k.trim().to_string()))
        .collect()
}

#[test]
fn every_enum_value_has_a_label_in_both_languages() {
    let schema = std::fs::read_to_string("schema.graphql").unwrap();
    let (en, fr) = (ftl_keys("en"), ftl_keys("fr"));
    let mut missing = Vec::new();
    for (kind, enums) in KINDS {
        for name in *enums {
            for value in enum_values(&schema, name) {
                let key = format!("enum-{}-{}", kind, value.to_lowercase().replace('_', "-"));
                for (lang, keys) in [("en", &en), ("fr", &fr)] {
                    if !keys.contains(&key) {
                        missing.push(format!("{} ({})", key, lang));
                    }
                }
            }
        }
    }
    assert!(missing.is_empty(), "missing enum labels: {:?}", missing);
}

#[test]
fn server_side_label_matches_fluent_and_falls_back_to_sentence_case() {
    assert_eq!(frontend::enum_label("domain", "DATA_ANALYTICS_AND_AI", "en"), "Data & AI");
    assert_eq!(frontend::enum_label("domain", "DATA_ANALYTICS_AND_AI", "fr"), "Données et IA");
    assert_eq!(frontend::enum_label("domain", "NOT_A_DOMAIN", "en"), "Not a domain");
}
