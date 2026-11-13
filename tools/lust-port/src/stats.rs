//! Counters keyed by feature name. Each key keeps, per file category, the
//! use count and the sets of files and functions that use it.

use crate::walk::Cat;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct Entry {
    pub count: [u64; 5],
    pub files: [HashSet<u32>; 5],
    pub fns: [HashSet<u32>; 5],
    /// Up to 3 `file:line` samples, prod first.
    pub samples: Vec<String>,
}

/// One function (fn item, impl fn, or trait fn with a body).
/// Only the id (index in Stats.fns) feeds the counts; the fields stay
/// for debugging and for parity with the inventory analyzer.
#[allow(dead_code)]
pub struct FnInfo {
    pub file: u32,
    pub cat: Cat,
    pub name: String,
    pub line: usize,
    /// Signature facts, used for the lifetime spread.
    pub sig: SigFacts,
}

#[derive(Default, Clone)]
#[allow(dead_code)]
pub struct SigFacts {
    pub explicit_lt_params: usize,
    pub ref_in_args: bool,
    pub ref_in_ret: bool,
    pub explicit_lt_use: bool,
    pub anon_lt_use: bool,
    pub static_lt_use: bool,
    /// Names of lifetime-carrying user types in the signature.
    pub lt_types: Vec<String>,
    pub type_params: usize,
    pub is_method: bool,
    pub self_ref: bool,
    pub self_mut: bool,
}

/// One unsupported-feature hit with a source position.
#[derive(Debug, Clone)]
pub struct Finding {
    pub feature: String,
    pub raw_key: String,
    pub file_idx: u32,
    pub byte_start: usize,
    pub byte_end: usize,
    pub line: usize,
    pub column: usize,
    pub snippet: String,
    /// active, test-only, or target-specific.
    pub scope: &'static str,
}

#[derive(Default)]
pub struct Stats {
    pub map: HashMap<String, Entry>,
    pub fns: Vec<FnInfo>,
    pub lines_by_cat: [usize; 5],
    pub files_by_cat: [usize; 5],
    /// Lines of `#[cfg(test)]` and `#[test]` items inside prod/gen files.
    pub inline_test_lines: [usize; 5],
    pub findings: Vec<Finding>,
    /// Dedup key: (feature id, file, byte start, byte end).
    pub seen_findings: HashSet<(String, u32, usize, usize)>,
}

impl Stats {
    #[allow(dead_code)] // small convenience wrapper kept from the analyzer
    pub fn add(&mut self, key: &str, cat: Cat, file: u32, func: Option<u32>, sample: impl FnOnce() -> String) {
        self.add_n(key, 1, cat, file, func, sample);
    }

    pub fn add_n(&mut self, key: &str, n: u64, cat: Cat, file: u32, func: Option<u32>, sample: impl FnOnce() -> String) {
        let e = match self.map.get_mut(key) {
            Some(e) => e,
            None => self.map.entry(key.to_string()).or_default(),
        };
        let c = cat as usize;
        e.count[c] += n;
        e.files[c].insert(file);
        if let Some(f) = func {
            e.fns[c].insert(f);
        }
        let want = e.samples.len() < 3 && (cat == Cat::Prod || e.samples.len() < 1);
        if want {
            e.samples.push(sample());
        }
    }
}
