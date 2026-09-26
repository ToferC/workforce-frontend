// Tera ignores macro imports inside an included partial: the including page
// must import every macro namespace the partial uses, or the page fails at
// render time. This checks every template statically — including pages no
// render test covers.

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::Path;

fn load(dir: &Path, root: &Path, out: &mut HashMap<String, String>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            load(&path, root, out);
        } else if path.extension().map_or(false, |e| e == "html") {
            let name = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
            out.insert(name, std::fs::read_to_string(&path).unwrap());
        }
    }
}

/// Namespaces called in `name` or in anything it includes.
fn used(name: &str, src: &HashMap<String, String>, call: &Regex, include: &Regex) -> HashSet<String> {
    let text = &src[name];
    let mut ns: HashSet<String> = call.captures_iter(text).map(|c| c[1].to_string()).collect();
    for inc in include.captures_iter(text) {
        ns.extend(used(&inc[1], src, call, include));
    }
    ns
}

#[test]
fn templates_import_every_macro_namespace_they_use() {
    let root = Path::new("templates");
    let mut src = HashMap::new();
    load(root, root, &mut src);

    let call = Regex::new(r"\b([a-z_]+)::[a-z_]+\(").unwrap();
    let include = Regex::new(r#"\{%-?\s*include\s+"([^"]+)""#).unwrap();
    let import = Regex::new(r#"\{%-?\s*import\s+"[^"]+"\s+as\s+(\w+)"#).unwrap();

    let mut missing = Vec::new();
    for (name, text) in &src {
        // Macro files are only ever imported, never rendered on their own.
        if name.starts_with("macros/") || name.ends_with("chart_macros.html") {
            continue;
        }
        let imported: HashSet<String> = import.captures_iter(text).map(|c| c[1].to_string()).collect();
        let mut gaps: Vec<String> = used(name, &src, &call, &include).difference(&imported).cloned().collect();
        if !gaps.is_empty() {
            gaps.sort();
            missing.push(format!("{} needs {:?}", name, gaps));
        }
    }
    missing.sort();
    assert!(missing.is_empty(), "missing macro imports:\n{}", missing.join("\n"));
}
