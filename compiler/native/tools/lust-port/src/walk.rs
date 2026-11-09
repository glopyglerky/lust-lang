//! Walks the module tree from cargo target roots. Follows `mod` items and
//! `#[path]`, evaluates cfg on module items for the scan target where it
//! can be decided syntactically, records the rest as undecidable. A file
//! that does not parse is kept (line count, error) and the walk goes on.

use crate::cfg_eval::{CfgEval, TargetInfo, Tri, eval_cfg_attr};
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Cat {
    Prod = 0,
    Gen = 1,
    Test = 2,
    Tool = 3,
    Orphan = 4,
}
pub const CATS: [Cat; 5] = [Cat::Prod, Cat::Gen, Cat::Test, Cat::Tool, Cat::Orphan];
impl Cat {
    pub fn name(self) -> &'static str {
        ["prod", "gen", "test", "tool", "orphan"][self as usize]
    }
}

#[allow(dead_code)] // cfg chain is recorded for debugging; scope() reads cfg_state
pub struct File {
    pub rel: String,
    pub path: PathBuf,
    pub text: String,
    pub lines: usize,
    pub ast: Option<syn::File>,
    pub cat: Cat,
    pub generator: Option<String>,
    /// Non-test cfg attributes on the mod chain, in source form.
    pub cfg: Vec<String>,
    /// Combined cfg state for this file on the scan target.
    pub cfg_state: Tri,
    pub parse_error: Option<String>,
    /// Effective Cargo features of the package this file belongs to.
    pub cargo_features: Arc<BTreeSet<String>>,
}

impl File {
    /// active, test-only, or target-specific (cfg is false on this target).
    pub fn scope(&self) -> &'static str {
        if self.cat == Cat::Test {
            "test-only"
        } else if self.cfg_state == Tri::False {
            "target-specific"
        } else {
            "active"
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // package, target name and kind name the root in errors and debugging
pub struct Root {
    pub path: PathBuf,
    pub cat: Cat,
    pub package: String,
    pub target_name: String,
    pub kind: String,
    /// Effective Cargo features of this root's package.
    pub cargo_features: Arc<BTreeSet<String>>,
}

#[derive(Debug, Clone)]
pub struct MissingMod {
    pub from_file: String,
    pub module: String,
    pub tried: Vec<String>,
}

#[allow(dead_code)] // roots stay available to callers and debugging
pub struct Files {
    pub files: Vec<File>,
    pub parse_errors: Vec<(String, String)>,
    pub missing_mods: Vec<MissingMod>,
    pub undecidable_cfg: Vec<String>,
    pub roots: Vec<Root>,
}

#[derive(Clone)]
struct Ctx {
    cat: Cat,
    cfg: Vec<String>,
    cfg_state: Tri,
    mod_dir: PathBuf,
    /// Scan target with this package's effective features set.
    target: TargetInfo,
    cargo_features: Arc<BTreeSet<String>>,
}

pub fn load(roots: Vec<Root>, base: &Path, target: &TargetInfo) -> Files {
    let mut out =
        Files { files: Vec::new(), parse_errors: Vec::new(), missing_mods: Vec::new(), undecidable_cfg: Vec::new(), roots: roots.clone() };
    let mut undecidable: BTreeSet<String> = BTreeSet::new();
    let mut by_path: HashMap<PathBuf, usize> = HashMap::new();
    let mut queue: VecDeque<(usize, Ctx)> = VecDeque::new();

    // Prod roots first so a file reached by prod and tests stays prod.
    let mut roots = roots;
    roots.sort_by_key(|r| r.cat);
    for r in &roots {
        let Some(idx) = add_file(&r.path, r.cat, &[], Tri::True, &r.cargo_features, base, &mut out, &mut by_path) else { continue };
        let mod_dir = r.path.parent().unwrap_or(base).to_path_buf();
        let pkg_target = target.with_features((*r.cargo_features).clone());
        queue.push_back((
            idx,
            Ctx {
                cat: r.cat,
                cfg: Vec::new(),
                cfg_state: Tri::True,
                mod_dir,
                target: pkg_target,
                cargo_features: r.cargo_features.clone(),
            },
        ));
    }

    while let Some((idx, ctx)) = queue.pop_front() {
        let (items, file_dir) = {
            let f = &out.files[idx];
            let Some(ast) = &f.ast else { continue };
            (ast.items.clone(), f.path.parent().map(|p| p.to_path_buf()))
        };
        let mut work: Vec<(Vec<syn::Item>, Ctx, Option<PathBuf>)> = vec![(items, ctx, file_dir)];
        while let Some((items, ctx, file_dir)) = work.pop() {
            for item in &items {
                let syn::Item::Mod(m) = item else { continue };
                let name = m.ident.to_string();
                let test = crate::analyze::is_cfg_test(&m.attrs);
                let mut child = ctx.clone();
                if test && (child.cat == Cat::Prod || child.cat == Cat::Gen) {
                    child.cat = Cat::Test;
                }
                for a in &m.attrs {
                    if a.path().is_ident("cfg") {
                        let ev: CfgEval = eval_cfg_attr(a, &ctx.target);
                        child.cfg_state = child.cfg_state.and(ev.state);
                        if ev.mentions_test && (child.cat == Cat::Prod || child.cat == Cat::Gen) {
                            child.cat = Cat::Test;
                        }
                        for u in &ev.undecidable {
                            undecidable.insert(format!("cfg({u})"));
                        }
                        if !ev.mentions_test || ev.mentions_target {
                            child.cfg.push(tokens_of(a));
                        }
                    } else if a.path().is_ident("cfg_attr") {
                        undecidable.insert(tokens_of(a));
                    }
                }
                let path_attr = path_attr_of(m);
                match &m.content {
                    Some((_, inner)) => {
                        let mut inline = child.clone();
                        inline.mod_dir = child.mod_dir.join(&name);
                        work.push((inner.clone(), inline, None));
                    }
                    None => {
                        let candidates: Vec<PathBuf> = match &path_attr {
                            Some(p) => vec![file_dir.clone().unwrap_or_else(|| child.mod_dir.clone()).join(p)],
                            None => vec![child.mod_dir.join(format!("{name}.rs")), child.mod_dir.join(&name).join("mod.rs")],
                        };
                        let mut found = None;
                        for c in &candidates {
                            let key = normalize(c);
                            if let Some(&old) = by_path.get(&key) {
                                found = Some((old, c.clone(), true));
                                break;
                            }
                            if c.exists() {
                                if let Some(id) =
                                    add_file(c, child.cat, &child.cfg, child.cfg_state, &child.cargo_features, base, &mut out, &mut by_path)
                                {
                                    found = Some((id, c.clone(), false));
                                }
                                break;
                            }
                        }
                        match found {
                            Some((child_idx, c, already)) => {
                                if !already {
                                    let fname = c.file_name().unwrap_or_default().to_string_lossy().into_owned();
                                    let child_dir = if fname == "mod.rs" || path_attr.is_some() {
                                        c.parent().unwrap_or(&child.mod_dir).to_path_buf()
                                    } else {
                                        c.with_extension("")
                                    };
                                    let mut next = child.clone();
                                    next.mod_dir = child_dir;
                                    queue.push_back((child_idx, next));
                                }
                            }
                            None => out.missing_mods.push(MissingMod {
                                from_file: out.files[idx].rel.clone(),
                                module: name,
                                tried: candidates.iter().map(|c| rel_of(c, base)).collect(),
                            }),
                        }
                    }
                }
            }
        }
    }

    for f in &mut out.files {
        if f.cat == Cat::Prod && f.generator.is_some() {
            f.cat = Cat::Gen;
        }
        if f.cat == Cat::Prod && f.rel.ends_with("_test.rs") {
            f.cat = Cat::Test;
        }
    }
    out.files.sort_by(|a, b| a.rel.cmp(&b.rel));
    out.undecidable_cfg = undecidable.into_iter().collect();
    out
}

#[allow(clippy::too_many_arguments)]
fn add_file(
    path: &Path,
    cat: Cat,
    cfg: &[String],
    cfg_state: Tri,
    cargo_features: &Arc<BTreeSet<String>>,
    base: &Path,
    out: &mut Files,
    by_path: &mut HashMap<PathBuf, usize>,
) -> Option<usize> {
    let key = normalize(path);
    if let Some(&i) = by_path.get(&key) {
        return Some(i);
    }
    let rel = rel_of(path, base);
    let (text, ast, err) = match std::fs::read_to_string(path) {
        Ok(text) => match syn::parse_file(&text) {
            Ok(ast) => (text, Some(ast), None),
            Err(e) => {
                let msg = e.to_string();
                out.parse_errors.push((rel.clone(), msg.clone()));
                (text, None, Some(msg))
            }
        },
        Err(e) => {
            let msg = format!("read failed: {e}");
            out.parse_errors.push((rel.clone(), msg.clone()));
            (String::new(), None, Some(msg))
        }
    };
    let idx = out.files.len();
    by_path.insert(key, idx);
    out.files.push(File {
        lines: text.lines().count(),
        generator: detect_generated(&text),
        rel,
        path: path.to_path_buf(),
        text,
        ast,
        cat,
        cfg: cfg.to_vec(),
        cfg_state,
        parse_error: err,
        cargo_features: cargo_features.clone(),
    });
    Some(idx)
}

fn tokens_of(a: &syn::Attribute) -> String {
    quote::quote!(#a).to_string().replace(' ', "")
}

fn path_attr_of(m: &syn::ItemMod) -> Option<String> {
    m.attrs.iter().find_map(|a| {
        if !a.path().is_ident("path") {
            return None;
        }
        if let syn::Meta::NameValue(nv) = &a.meta {
            if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                return Some(s.value());
            }
        }
        None
    })
}

pub fn rel_of(path: &Path, base: &Path) -> String {
    let p = normalize(path);
    let b = normalize(base);
    match p.strip_prefix(&b) {
        Ok(r) => r.to_string_lossy().replace('\\', "/"),
        Err(_) => p.to_string_lossy().replace('\\', "/"),
    }
}

/// Absolute path with `.` and `..` removed lexically (no fs access, no
/// symlink chase, so read-only trees and missing files are both fine).
pub fn normalize(path: &Path) -> PathBuf {
    let abs = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let mut out = PathBuf::new();
    for c in abs.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

fn detect_generated(text: &str) -> Option<String> {
    for line in text.lines().take(6) {
        let l = line.trim();
        if !l.starts_with("//") {
            continue;
        }
        let low = l.to_lowercase();
        let hit = low.contains("@generated")
            || low.contains("code generated by")
            || low.contains("do not edit")
            || low.starts_with("//! generated by")
            || low.ends_with(". generated.");
        if hit {
            return Some(l.trim_start_matches('/').trim_start_matches('!').trim().to_string());
        }
    }
    None
}
