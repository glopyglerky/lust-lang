// The unit memo (plan item A, approved 07:57): what `cc::build` needs of a unit whose text
// it is not given. A persistent session (a daemon or a front image) fingerprints each
// release unit's inputs (`UnitIn::fp`) and passes no text for a unit whose fingerprint is
// one the last build made (under any unit name: names are positions); the memo maps it to the
// unit's key (the exact text key: compiler, flags, target, profile part and text, so the
// object cache stays exact), so the session neither assembles nor hashes the unit.
//
// The memo is a file in the session's cc work dir (`unit-memo`), written by every build
// that was given fingerprints, with the units of that build only (one build deep). A
// release cc job can run after the session image is saved (DEFER_CC), so the memo cannot
// live in the session's heap. A wrong fingerprint can only reuse the last build's key
// for a unit, never a key no text gave: LUST_CHECK_UNIT_FP=1 (front-check, cache_check
// --release --front) passes every text and fails a build whose text key differs.
use std::collections::HashMap;
use std::path::Path;

/// A unit for `cc::build`: its name, its text, and the session's fingerprint of its inputs.
/// Without a fingerprint the unit is not memoized; without text, its fingerprint must be in
/// the memo, with its object in the cache (else `CcError::NeedText`).
pub struct UnitIn {
    pub name: String,
    pub fp: Option<u128>,
    pub text: Option<String>,
}

impl UnitIn {
    /// A unit with its text and no fingerprint (one-shot builds, tests).
    pub fn text(name: impl Into<String>, text: String) -> UnitIn {
        UnitIn { name: name.into(), fp: None, text: Some(text) }
    }
}

#[derive(Debug)]
pub enum CcError {
    /// Call again with the texts of these units: the memo or the object cache lacks them.
    NeedText(Vec<String>),
    Failed(String),
}

impl std::fmt::Display for CcError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CcError::NeedText(names) => write!(f, "internal: the C build needs the text of {} units ({})", names.len(), names.join(", ")),
            CcError::Failed(e) => f.write_str(e),
        }
    }
}

impl From<String> for CcError {
    fn from(e: String) -> CcError {
        CcError::Failed(e)
    }
}

/// What `cc::build` keeps of a unit.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Entry {
    pub name: String,
    pub key: String,
    /// The file name it compiles as: its PGO name with `.c` or `.ll`.
    pub file: String,
    /// Its text length (largest units compile first).
    pub len: usize,
}

fn path(work: &Path) -> std::path::PathBuf {
    work.join("unit-memo")
}

/// The entries of the memo in `work`, if the build that wrote it had this `tag` (what keys
/// hold besides the text); none otherwise.
pub(super) fn load(work: &Path, tag: &str) -> HashMap<u128, Entry> {
    let Ok(text) = std::fs::read_to_string(path(work)) else { return HashMap::new() };
    let mut lines = text.lines();
    if lines.next() != Some(tag) {
        return HashMap::new();
    }
    lines
        .filter_map(|l| {
            let mut f = l.split('\t');
            let fp = u128::from_str_radix(f.next()?, 16).ok()?;
            let (name, key, file) = (f.next()?.to_string(), f.next()?.to_string(), f.next()?.to_string());
            let len = f.next()?.parse().ok()?;
            Some((fp, Entry { name, key, file, len }))
        })
        .collect()
}

/// Replaces the memo in `work` (written aside, then renamed: a reader sees the old file or
/// the new one).
pub(super) fn save(work: &Path, tag: &str, entries: &[(u128, Entry)]) -> Result<(), String> {
    let mut text = format!("{tag}\n");
    for (fp, e) in entries {
        text.push_str(&format!("{fp:032x}\t{}\t{}\t{}\t{}\n", e.name, e.key, e.file, e.len));
    }
    let tmp = work.join(format!("unit-memo.{}", std::process::id()));
    std::fs::write(&tmp, text).and_then(|_| std::fs::rename(&tmp, path(work))).map_err(|e| format!("cannot write {}: {e}", path(work).display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_memo_reads_back_only_under_its_tag() {
        let work = std::env::temp_dir().join(format!("lust-memo-test-{}", std::process::id()));
        std::fs::create_dir_all(&work).unwrap();
        let e = Entry { name: "u7".into(), key: "ab".repeat(16), file: "x0123.c".into(), len: 42 };
        save(&work, "tag-a", &[(7u128 << 100 | 5, e.clone())]).unwrap();
        assert_eq!(load(&work, "tag-a").get(&(7u128 << 100 | 5)), Some(&e));
        assert!(load(&work, "tag-b").is_empty(), "another compiler, flag set, profile or lust build drops the memo");
        let _ = std::fs::remove_dir_all(&work);
    }
}
