//! The feature table: which Rust features lust supports, and in which
//! PLAN.md milestone the rest arrive. Loaded from `features.json` next to
//! this crate (embedded at build time). `--features <file>` replaces the
//! table; `--capabilities <file>` overrides status and milestone by id, so
//! a later `lust capabilities --json` can feed the same ids.

use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;

pub const EMBEDDED: &str = include_str!("../features.json");

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // inventory_step is kept for the step table in features.json
pub struct Milestone {
    pub n: i64,
    pub label: String,
    pub inventory_step: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Feature {
    pub id: String,
    pub title: String,
    /// One of: supported, partial, unsupported.
    pub status: String,
    /// PLAN.md milestone that adds this feature. 0 means supported today.
    pub milestone: Option<i64>,
    /// Step id in INVENTORY.md section 5 ("0".."10", "7a", "7b"), if any.
    pub inventory_step: Option<String>,
    pub root_cause: String,
    /// One of: compiler, adapter, mechanical, manual.
    pub fix_class: String,
    pub release_safe_rewrite: bool,
    pub release_impact: String,
    pub notes: String,
    /// Raw keys that create one finding each.
    #[serde(default)]
    pub primary_keys: Vec<String>,
    /// Raw keys that make a file need this feature (coverage).
    #[serde(default)]
    pub keys: Vec<String>,
}

impl Feature {
    pub fn is_gap(&self) -> bool {
        self.status != "supported"
    }
    pub fn fix_label(&self) -> &'static str {
        match self.fix_class.as_str() {
            "compiler" => "compiler work",
            "adapter" => "adapter",
            "mechanical" => "mechanical rewrite",
            "manual" => "manual",
            _ => "manual",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // schema_version guards future table formats
pub struct FeatureTable {
    pub schema_version: u32,
    #[serde(default)]
    pub milestones: Vec<Milestone>,
    pub features: Vec<Feature>,
    #[serde(skip)]
    index: HashMap<String, usize>,
    /// Raw key to feature index. The visitor calls classify() once per
    /// hit (millions of times on ts-rust) but distinct keys are few.
    #[serde(skip)]
    cache: RefCell<HashMap<String, Option<usize>>>,
}

impl FeatureTable {
    pub fn embedded() -> Result<FeatureTable, String> {
        Self::parse(EMBEDDED)
    }

    pub fn parse(text: &str) -> Result<FeatureTable, String> {
        let mut t: FeatureTable = serde_json::from_str(text).map_err(|e| format!("bad features.json: {e}"))?;
        t.rebuild_index();
        Ok(t)
    }

    fn rebuild_index(&mut self) {
        self.index = self.features.iter().enumerate().map(|(i, f)| (f.id.clone(), i)).collect();
        self.cache.borrow_mut().clear();
    }

    pub fn get(&self, id: &str) -> Option<&Feature> {
        self.index.get(id).map(|&i| &self.features[i])
    }

    pub fn milestone_label(&self, n: Option<i64>) -> String {
        match n {
            None => "no milestone".to_string(),
            Some(n) => self
                .milestones
                .iter()
                .find(|m| m.n == n)
                .map(|m| format!("milestone {}: {}", n, m.label))
                .unwrap_or_else(|| format!("milestone {n}")),
        }
    }

    /// First feature whose `keys` match this raw scanner key.
    pub fn classify(&self, raw_key: &str) -> Option<&Feature> {
        if let Some(v) = self.cache.borrow().get(raw_key) {
            return v.map(|i| &self.features[i]);
        }
        // `use std::sync::Arc;` yields std.use.* where a type use yields
        // std.type.*. Try the type form first so a use-only file blocks
        // on the right milestone instead of the catch-all rule.
        let alt = [("std.use.", "std.type."), ("ext.use.", "ext.type.")]
            .iter()
            .find_map(|(a, b)| raw_key.strip_prefix(a).map(|rest| format!("{b}{rest}")));
        let mut idx = alt.as_ref().and_then(|alt| self.features.iter().position(|f| f.keys.iter().any(|p| glob_match(p, alt))));
        if idx.is_none() {
            idx = self.features.iter().position(|f| f.keys.iter().any(|p| glob_match(p, raw_key)));
        }
        self.cache.borrow_mut().insert(raw_key.to_string(), idx);
        idx.map(|i| &self.features[i])
    }

    pub fn is_primary(f: &Feature, raw_key: &str) -> bool {
        f.primary_keys.iter().any(|p| glob_match(p, raw_key))
    }

    /// Applies a capabilities file: {"features":[{"id":..,"status":..,"milestone":..}]}
    /// Unknown ids are reported, known ids override status and milestone.
    pub fn apply_capabilities(&mut self, text: &str) -> Result<Vec<String>, String> {
        let v: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("bad capabilities file: {e}"))?;
        let arr = v.get("features").and_then(|x| x.as_array()).ok_or("capabilities file has no features array")?;
        let mut unknown = Vec::new();
        for item in arr {
            let Some(id) = item.get("id").and_then(|x| x.as_str()) else { continue };
            match self.index.get(id).copied() {
                Some(i) => {
                    self.cache.borrow_mut().clear();
                    if let Some(s) = item.get("status").and_then(|x| x.as_str()) {
                        self.features[i].status = s.to_string();
                    }
                    if let Some(m) = item.get("milestone") {
                        self.features[i].milestone = m.as_i64();
                    }
                }
                None => unknown.push(id.to_string()),
            }
        }
        Ok(unknown)
    }
}

/// Glob match where `*` matches any run of characters (including empty).
pub fn glob_match(pat: &str, text: &str) -> bool {
    let p: Vec<char> = pat.chars().collect();
    let t: Vec<char> = text.chars().collect();
    // Iterative two-pointer glob with one star backtrack point.
    let (mut pi, mut ti) = (0usize, 0usize);
    let (mut star, mut mark) = (None::<usize>, 0usize);
    while ti < t.len() {
        if pi < p.len() && p[pi] == t[ti] {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            pi += 1;
            mark = ti;
        } else if let Some(s) = star {
            pi = s + 1;
            mark += 1;
            ti = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_cases() {
        assert!(glob_match("closure", "closure"));
        assert!(!glob_match("closure", "closure.move"));
        assert!(glob_match("closure.*", "closure.move"));
        assert!(glob_match("lt.ref_pos.*.elided", "lt.ref_pos.arg.elided"));
        assert!(!glob_match("lt.ref_pos.*.elided", "lt.ref_pos.arg.static"));
        assert!(glob_match("ext.type.rustix::*", "ext.type.rustix::fs::File"));
        assert!(glob_match("gen.type_param*", "gen.type_param.fn"));
        assert!(glob_match("*", "anything"));
        assert!(!glob_match("attr.cfg", "attr.cfg_attr"));
    }

    #[test]
    fn classify_order() {
        let t = FeatureTable::embedded().unwrap();
        assert_eq!(t.classify("lt.ref.elided").unwrap().id, "supported.baseline");
        assert_eq!(t.classify("closure").unwrap().id, "closures.general");
        assert_eq!(t.classify("type.dyn").unwrap().id, "dyn.trait");
        assert_eq!(t.classify("dyn.std::any::Any").unwrap().id, "dyn.any");
        assert_eq!(t.classify("impl.trait.std::ops::Drop").unwrap().id, "core.drop_impl");
        assert_eq!(t.classify("lt.ref.static").unwrap().id, "lifetimes.static_anon");
        assert_eq!(t.classify("lt.ref.named").unwrap().id, "lifetimes.named");
        assert_eq!(t.classify("derive.Clone").unwrap().status, "supported");
        assert_eq!(t.classify("derive.Default").unwrap().id, "core.derives_extra");
        assert_eq!(t.classify("item.mod.path_attr").unwrap().id, "modules.path_attr");
        assert_eq!(t.classify("std.use.std::sync::Arc").unwrap().id, "threads.sync");
        assert!(t.classify("method.len").is_none());
        // Split features added by the port-scan-sync probes.
        assert_eq!(t.classify("macro.thread_local").unwrap().id, "threads.thread_local");
        assert_eq!(t.classify("expr.unsafe").unwrap().id, "unsafe.blocks");
        // The cinterop split (w-cinterop).
        assert_eq!(t.classify("type.raw_ptr").unwrap().id, "cinterop.raw_ptr");
        assert_eq!(t.classify("item.foreign_mod").unwrap().id, "cinterop.extern");
        assert_eq!(t.classify("fn.extern_abi").unwrap().id, "cinterop.extern");
        assert_eq!(t.classify("attr.global_allocator").unwrap().id, "cinterop.global_allocator");
        assert_eq!(t.classify("ext.type.rustix::fs::FileType").unwrap().id, "ext.rustix");
        assert_eq!(t.classify("attr.cfg").unwrap().id, "attrs.cfg");
        assert_eq!(t.classify("attr.default").unwrap().id, "attrs.cfg");
        assert_eq!(t.classify("attr.rustfmt::skip").unwrap().id, "attrs.cfg");
        assert_eq!(t.classify("attr.ignore").unwrap().id, "attrs.cfg");
        assert_eq!(t.classify("attr.should_panic").unwrap().id, "attrs.cfg");
        assert_eq!(t.classify("attr.serde").unwrap().id, "attrs.serde");
        assert_eq!(t.classify("attr.used").unwrap().id, "attrs.unsupported");
        assert_eq!(t.classify("attr.track_caller").unwrap().id, "attrs.track_caller");
        assert_eq!(t.classify("attr.non_exhaustive").unwrap().id, "attrs.non_exhaustive");
        assert_eq!(t.classify("dyn.std::ops::Fn").unwrap().id, "dyn.fn");
        assert_eq!(t.classify("dyn.std::io::Write").unwrap().id, "dyn.std");
        assert_eq!(t.classify("dyn_in.Box").unwrap().id, "dyn.boxed");
        assert_eq!(t.classify("dyn_in.&").unwrap().id, "dyn.trait");
        assert_eq!(t.classify("type.impl_trait.ret").unwrap().id, "traits.impl_trait_ret");
        assert_eq!(t.classify("std.type.std::rc::Rc").unwrap().id, "std.rc");
        assert_eq!(t.classify("std.type.std::rc::Weak").unwrap().id, "std.rc_refcell");
        assert_eq!(t.classify("std.type.std::borrow::Cow").unwrap().id, "std.cow");
        assert_eq!(t.classify("std.type.std::borrow::ToOwned").unwrap().id, "supported.std_core");
        assert_eq!(t.classify("type.fn_ptr.lifetime").unwrap().id, "closures.fn_ptr_lifetime");
        assert_eq!(t.classify("map.hasher_arg").unwrap().id, "std.hashmap_hasher");
        assert_eq!(t.classify("std.type.std::collections::HashMap").unwrap().id, "std.hashmap");
        assert_eq!(t.classify("macro.write").unwrap().id, "macros.std_unsupported");
        assert_eq!(t.classify("macro.matches").unwrap().id, "macros.std_missing");
        // The serde split (w-tail): per-crate rows beat the catch-alls.
        assert_eq!(t.classify("ext.use.serde::Deserialize").unwrap().id, "ext.serde");
        assert_eq!(t.classify("derive.Deserialize").unwrap().id, "ext.serde");
        assert_eq!(t.classify("derive.Serialize").unwrap().id, "ext.serde");
        assert_eq!(t.classify("derive.Builder").unwrap().id, "derives.unknown");
        assert_eq!(t.classify("ext.expr.serde_json::from_str").unwrap().id, "ext.serde_json");
        assert_eq!(t.classify("ext.expr.anyhow::Ok").unwrap().id, "ext.unknown_crate");
    }

    #[test]
    fn capabilities_override() {
        let mut t = FeatureTable::embedded().unwrap();
        let unknown = t
            .apply_capabilities(
                r#"{"features":[{"id":"closures.general","status":"supported","milestone":0},{"id":"nope.nope","status":"supported"}]}"#,
            )
            .unwrap();
        assert_eq!(unknown, vec!["nope.nope".to_string()]);
        assert_eq!(t.get("closures.general").unwrap().status, "supported");
        assert!(!t.get("closures.general").unwrap().is_gap());
    }
}
