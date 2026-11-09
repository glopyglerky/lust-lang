//! Effective Cargo features for one scanned package, under the project
//! dev config (docs/dev-features.md in the lust repo): the same
//! `dev-features` / `dev-exclude-features` keys lust reads from
//! `[package.metadata.lust]`, `[workspace.metadata.lust]`, or a
//! `.lust.toml` file. With no config the package defaults apply, as a
//! plain Cargo build would. Command line overrides in main.rs replace
//! the config for one scan.

use crate::cargo_meta::Package;
use std::collections::{BTreeSet, HashSet};
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct DevConfig {
    pub dev_features: Option<Vec<String>>,
    pub dev_exclude_features: Option<Vec<String>>,
}

impl DevConfig {
    fn any(&self) -> bool {
        self.dev_features.is_some() || self.dev_exclude_features.is_some()
    }
}

/// One key of a `[*.metadata.lust]` table in cargo metadata JSON.
fn from_metadata(value: &serde_json::Value) -> DevConfig {
    let lust = value.get("lust");
    let list = |key: &str| {
        lust.and_then(|l| l.get(key))
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect::<Vec<String>>())
    };
    DevConfig { dev_features: list("dev-features"), dev_exclude_features: list("dev-exclude-features") }
}

/// Quoted strings in one TOML array value, as in lust's protocol.rs.
fn toml_str_array(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c == '"' || c == '\'' {
            let mut s = String::new();
            for d in chars.by_ref() {
                if d == c {
                    break;
                }
                s.push(d);
            }
            out.push(s);
        }
        if c == ']' {
            break;
        }
    }
    out
}

/// Top-level array keys of a `.lust.toml` text.
fn from_dotfile(text: &str) -> DevConfig {
    let mut cfg = DevConfig::default();
    let mut lines = text.lines().peekable();
    let mut top = true;
    while let Some(raw) = lines.next() {
        let line = raw.trim();
        if line.starts_with('[') {
            top = false;
            continue;
        }
        if !top {
            continue;
        }
        let key = line.split('=').next().unwrap_or("").trim();
        if key != "dev-features" && key != "dev-exclude-features" {
            continue;
        }
        let mut value = line.split_once('=').map_or("", |(_, v)| v).to_string();
        while !value.contains(']') {
            match lines.next() {
                Some(next) => {
                    value.push(' ');
                    value.push_str(next.trim());
                }
                None => break,
            }
        }
        if key == "dev-features" {
            cfg.dev_features = Some(toml_str_array(&value));
        } else {
            cfg.dev_exclude_features = Some(toml_str_array(&value));
        }
    }
    cfg
}

/// The dev config for one package: package metadata, then workspace
/// metadata, then the nearest `.lust.toml` from the package directory
/// up to the workspace root.
pub fn config_for(pkg: &Package, workspace_metadata: &serde_json::Value, workspace_root: &Path) -> DevConfig {
    let cfg = from_metadata(&pkg.metadata);
    if cfg.any() {
        return cfg;
    }
    let cfg = from_metadata(workspace_metadata);
    if cfg.any() {
        return cfg;
    }
    let mut dir = pkg.manifest_path.parent();
    while let Some(d) = dir {
        if let Ok(text) = std::fs::read_to_string(d.join(".lust.toml")) {
            let cfg = from_dotfile(&text);
            if cfg.any() {
                return cfg;
            }
        }
        if d == workspace_root {
            break;
        }
        dir = d.parent();
    }
    DevConfig::default()
}

/// What one feature selection turns on: the activated feature names
/// (what `cfg(feature = ...)` sees) and the optional dependency names
/// it enables (through `dep:` entries and implicit features).
#[derive(Debug, Default)]
pub struct Effective {
    pub features: BTreeSet<String>,
    pub deps: HashSet<String>,
}

/// Expands a selected feature list through the package feature table.
/// Entries are plain feature names, `dep:<name>`, `<dep>/<feature>`, or
/// `<dep>?/<feature>` (weak). Only plain names name a cfg feature of
/// this package. The expansion is syntactic: cargo metadata here runs
/// with `--no-deps`, so no resolve graph is read.
pub fn expand(pkg: &Package, selected: &[String]) -> Effective {
    let mut out = Effective::default();
    // Optional deps referenced with `dep:` lose their implicit feature.
    let mut dep_colon: HashSet<String> = HashSet::new();
    for vals in pkg.features.values() {
        for v in vals {
            if let Some(d) = v.strip_prefix("dep:") {
                dep_colon.insert(d.to_string());
            }
        }
    }
    let mut queue: Vec<String> = selected.to_vec();
    while let Some(name) = queue.pop() {
        if name.contains(':') || name.contains('/') {
            continue;
        }
        if !out.features.insert(name.clone()) {
            continue;
        }
        // An implicit feature for an optional dependency enables it.
        if pkg.dependencies.iter().any(|d| d.optional && d.name == name && !dep_colon.contains(&name)) {
            out.deps.insert(name.clone());
        }
        if let Some(vals) = pkg.features.get(&name) {
            for v in vals {
                if let Some(d) = v.strip_prefix("dep:") {
                    out.deps.insert(d.to_string());
                } else if let Some((dep, _)) = v.split_once('/') {
                    if let Some(d) = dep.strip_suffix('?') {
                        let _ = d; // weak: enables nothing by itself
                    }
                } else {
                    queue.push(v.clone());
                }
            }
        }
    }
    out
}

/// The effective features of one package. `cli_features` (exact list)
/// and `cli_exclude` come from the scanner command line and replace
/// the project config when set.
pub fn effective(
    pkg: &Package,
    workspace_metadata: &serde_json::Value,
    workspace_root: &Path,
    cli_features: Option<&[String]>,
    cli_exclude: Option<&[String]>,
) -> Effective {
    let defaults: Vec<String> = pkg.features.get("default").cloned().unwrap_or_default();
    let minus = |ex: &[String]| defaults.iter().filter(|f| !ex.contains(f)).cloned().collect::<Vec<String>>();
    let selected = if let Some(list) = cli_features {
        list.to_vec()
    } else if let Some(ex) = cli_exclude {
        minus(ex)
    } else {
        let cfg = config_for(pkg, workspace_metadata, workspace_root);
        if let Some(list) = cfg.dev_features {
            list
        } else if let Some(ex) = cfg.dev_exclude_features {
            minus(&ex)
        } else {
            defaults
        }
    };
    expand(pkg, &selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn pkg() -> Package {
        let mut features = BTreeMap::new();
        features.insert("default".to_string(), vec!["jemalloc".to_string(), "fast".to_string()]);
        features.insert("jemalloc".to_string(), vec!["dep:tikv-jemallocator".to_string(), "dep:tikv-jemalloc-ctl".to_string()]);
        features.insert("fast".to_string(), vec![]);
        features.insert("noembed".to_string(), vec!["goport_util/noembed".to_string()]);
        Package {
            name: "ts_goport".into(),
            version: "0.0.0".into(),
            id: "x".into(),
            manifest_path: "/tmp/nope/Cargo.toml".into(),
            targets: vec![],
            dependencies: vec![],
            edition: "2024".into(),
            features,
            metadata: serde_json::json!({"lust": {"dev-exclude-features": ["jemalloc"]}}),
        }
    }

    #[test]
    fn exclude_jemalloc_leaves_fast() {
        let p = pkg();
        let eff = effective(&p, &serde_json::Value::Null, Path::new("/tmp"), None, None);
        assert_eq!(eff.features, BTreeSet::from(["fast".to_string()]));
        assert!(eff.deps.is_empty(), "{:?}", eff.deps);
        // No config: defaults with jemalloc and its deps.
        let mut p2 = pkg();
        p2.metadata = serde_json::Value::Null;
        let eff = effective(&p2, &serde_json::Value::Null, Path::new("/tmp"), None, None);
        assert!(eff.features.contains("jemalloc") && eff.deps.contains("tikv-jemalloc-ctl"));
        // CLI exact list wins over the config.
        let eff = effective(&p, &serde_json::Value::Null, Path::new("/tmp"), Some(&["noembed".to_string()]), None);
        assert_eq!(eff.features, BTreeSet::from(["noembed".to_string()]));
    }
}
