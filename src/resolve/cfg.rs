// cfg evaluation and stripping (docs/ARCH.md section 1.6).
//
// The cfg environment is a fixed set per crate session: target values from a
// small built-in table, per-unit features, and the profile flags. Predicates
// arrive as the raw token trees an attribute stored (`#[cfg(...)]` contents,
// or a Cargo dependency's target string re-lexed by cargo.rs). Evaluation is
// total: a known name with an unknown value is false, an unknown *name* is
// error L0520, because silently dropping a module would change the program.
use crate::ast::{Attr, AttrInput};
use crate::diag::{Diag, Span};
use crate::intern::Interner;
use crate::lexer::{Tok, TokenTree};
use crate::protocol::ProfileFlags;
use std::collections::BTreeSet;

/// Every cfg name lust knows (the rustc well-known list of section 1.6, plus
/// `feature`). Any other name in a predicate is L0520. `lust_dev` is known
/// only for shipped crates; in user code it takes the unknown-name error.
pub const KNOWN_CFG_NAMES: &[&str] = &[
    "all",
    "any",
    "not",
    "feature",
    "target_arch",
    "target_os",
    "target_vendor",
    "target_family",
    "target_env",
    "target_abi",
    "target_endian",
    "target_pointer_width",
    "target_has_atomic",
    "target_has_atomic_primitive_alignment",
    "target_feature",
    "target_thread_local",
    "unix",
    "windows",
    "panic",
    "overflow_checks",
    "debug_assertions",
    "test",
    "docsrs",
    "fuzzing",
    "miri",
    "target_os_version",
    "target_abi_version",
];

/// Cfg names rustc 1.99 gates behind a nightly feature (E0658). A
/// predicate that names one is L0523 wherever it appears, as in rustc,
/// even under `any`/`all`/`not` that would not need its value.
pub const UNSTABLE_CFG_NAMES: &[&str] = &[
    "overflow_checks",
    "target_has_atomic_load_store",
    "target_thread_local",
    "ub_checks",
    "contract_checks",
    "sanitize",
    "relocation_model",
    "fmt_debug",
    "sanitizer_cfi_generalize_pointers",
    "sanitizer_cfi_normalize_integers",
    "target_has_reliable_f16",
    "target_has_reliable_f16_math",
    "target_has_reliable_f128",
    "target_has_reliable_f128_math",
];

/// The L0523 error for a cfg name rustc gates as unstable, or None.
pub fn unstable_cfg(name: &str, span: Span) -> Option<Diag> {
    UNSTABLE_CFG_NAMES.contains(&name).then(|| {
        Diag::new(
            "L0523",
            span,
            format!("`cfg({name})` is unstable in Rust"),
            "Remove the predicate: stable Rust rejects it without a nightly feature.",
        )
    })
}

/// One crate session's cfg environment. Flags are names that are on
/// (`unix`); pairs are `name = "value"` facts (`target_os = "linux"`). A pair
/// name may repeat (`target_family`, `target_feature`, `target_has_atomic`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CfgEnv {
    pub flags: BTreeSet<String>,
    pub pairs: Vec<(String, String)>,
    pub features: BTreeSet<String>,
    /// Custom cfg names declared by build-script output (recorded by
    /// `lust port prepare`). They are known names; their values here are
    /// flags or pairs like the rest.
    pub custom_names: BTreeSet<String>,
    /// Additions owned by this crate, separated from target/profile facts so
    /// they can be dropped when constructing a dependency's environment.
    custom_flags: BTreeSet<String>,
    custom_pair_start: Option<usize>,
    /// True only while a shipped std or adapter crate resolves: `lust_dev`
    /// is then a known, true flag (section 1.6).
    pub allow_lust_dev: bool,
}

impl CfgEnv {
    /// This environment with another feature set and all other facts intact.
    /// A different crate uses `for_dependency` to discard crate-local facts.
    pub fn with_features(&self, features: &[String]) -> CfgEnv {
        let mut env = self.clone();
        env.features = features.iter().cloned().collect();
        env
    }

    /// A dependency inherits the target and profile, but not this crate's
    /// test harness, cfg declarations, or build-script cfg values. Its own
    /// declarations and values are added after this step.
    pub fn for_dependency(&self, features: &[String]) -> CfgEnv {
        let mut env = self.with_features(features);
        for flag in std::mem::take(&mut env.custom_flags) {
            env.flags.remove(&flag);
        }
        if let Some(start) = env.custom_pair_start.take() {
            env.pairs.truncate(start);
        }
        env.custom_names.clear();
        env.flags.remove("test");
        env.allow_lust_dev = false;
        env
    }

    /// The environment for one target triple (empty means the host), one
    /// per-unit feature set, the test-crate flag, and the profile. Tier 1
    /// targets and `wasm32-wasip1` (classification only) are in the table;
    /// another triple gets the parts its name proves (arch, pointer width
    /// for `wasm32`/`64` suffix arches) and nothing else, never a guess at
    /// a value that would change code silently beyond what the name says.
    pub fn new(triple: &str, features: &[String], test: bool, profile: ProfileFlags) -> CfgEnv {
        let triple = if triple.is_empty() { crate::protocol::host_triple() } else { triple };
        let mut env = CfgEnv { features: features.iter().cloned().collect(), ..Default::default() };
        let parts: Vec<&str> = triple.split('-').collect();
        let arch = parts.first().copied().unwrap_or("");
        env.pair("target_arch", arch);
        env.pair(
            "target_endian",
            if matches!(arch, "s390x" | "powerpc" | "powerpc64" | "sparc64" | "mips" | "mips64") { "big" } else { "little" },
        );
        env.pair("target_pointer_width", if arch.contains("64") || arch == "wasm64" { "64" } else { "32" });
        // vendor-os[-env[-abi]] after the arch, when the triple has them.
        // Two-part triples (`wasm32-wasip1`) carry the os in the vendor
        // slot; rustc reports vendor "unknown" for them.
        let (vendor_os, env_abi_at) = if parts.len() == 2 {
            (Some(("unknown", parts[1])), 2usize)
        } else {
            (parts.get(1).zip(parts.get(2)).map(|(&v, &o)| (v, o)), 3usize)
        };
        let mut os_name = "";
        let mut env_part = "";
        if let Some((vendor, os)) = vendor_os {
            env.pair("target_vendor", vendor);
            let (name, family) = match os {
                "darwin" => ("macos", "unix"),
                "windows" => ("windows", "windows"),
                "wasip1" | "wasip2" | "wasi" => ("wasi", "wasm"),
                "emscripten" => ("emscripten", "unix"),
                other => (other, "unix"),
            };
            os_name = name;
            env.pair("target_os", os_name);
            env.pair("target_family", family);
            if family == "unix" {
                env.flags.insert("unix".into());
            }
            if family == "windows" {
                env.flags.insert("windows".into());
            }
            env_part = parts.get(env_abi_at).copied().unwrap_or("");
            // `gnu`/`msvc`/`musl` may carry an ABI suffix (`gnueabi`); keep the
            // written value, and the abi as its own pair only when a 5th part
            // exists (for example `x86_64-pc-windows-msvc` has empty abi).
            env.pair("target_env", env_part);
            env.pair("target_abi", parts.get(env_abi_at + 1).copied().unwrap_or(""));
        }
        // Atomics and target features for the release triples, copied from
        // `rustc --print cfg` (1.99) per triple. Width 128 exists where
        // rustc lists it (aarch64 everywhere here, x86_64 on macOS and
        // Windows, not on Linux). Another triple keeps the old base row:
        // widths to 64 and the x86_64 baseline features, classification
        // only, as the doc comment above says.
        if arch != "wasm32" {
            for w in ["8", "16", "32", "64", "ptr"] {
                env.pair("target_has_atomic", w);
                env.pair("target_has_atomic_primitive_alignment", w);
            }
            if arch == "aarch64" || (arch == "x86_64" && matches!(os_name, "macos" | "windows")) {
                env.pair("target_has_atomic", "128");
                env.pair("target_has_atomic_primitive_alignment", "128");
            }
        }
        let features: &[&str] = match (arch, os_name, env_part) {
            ("x86_64", "linux", "musl") => &["crt-static", "fxsr", "sse", "sse2"],
            ("aarch64", "linux", "musl") => &["crt-static", "neon"],
            ("aarch64", "linux", _) => &["neon"],
            ("x86_64", "macos", _) => &["cmpxchg16b", "fxsr", "sse", "sse2", "sse3", "sse4.1", "ssse3"],
            ("aarch64", "macos", _) => &[
                "aes", "crc", "dit", "dotprod", "dpb", "dpb2", "fcma", "fhm", "flagm", "fp16", "frintts", "jsconv", "lor",
                "lse", "neon", "paca", "pacg", "pan", "pmuv3", "ras", "rcpc", "rcpc2", "rdm", "sb", "sha2", "sha3", "ssbs",
                "vh",
            ],
            ("x86_64", "windows", _) => &["cmpxchg16b", "fxsr", "sse", "sse2", "sse3"],
            ("aarch64", "windows", _) => &["neon"],
            ("x86_64", _, _) => &["fxsr", "sse", "sse2"],
            _ => &[],
        };
        for f in features {
            env.pair("target_feature", f);
        }
        if !arch.starts_with("wasm") {
            env.flags.insert("target_thread_local".into());
        }
        env.pair("panic", if profile.panic_abort { "abort" } else { "unwind" });
        if profile.overflow_checks {
            env.flags.insert("overflow_checks".into());
        }
        if profile.debug_assertions {
            env.flags.insert("debug_assertions".into());
        }
        if test {
            env.flags.insert("test".into());
        }
        env
    }

    /// True for the release triples whose full cfg table above is copied
    /// from rustc (the six ts-rust release targets, both Linux libc
    /// flavors, and x86_64 windows-gnu). A foreign `--target` names one
    /// of these or it stays a clear error: for another triple the table
    /// is classification only and a check could accept where rustc
    /// rejects.
    pub fn is_release_target(triple: &str) -> bool {
        matches!(
            triple,
            "x86_64-unknown-linux-gnu"
                | "x86_64-unknown-linux-musl"
                | "aarch64-unknown-linux-gnu"
                | "aarch64-unknown-linux-musl"
                | "aarch64-apple-darwin"
                | "x86_64-apple-darwin"
                | "x86_64-pc-windows-msvc"
                | "aarch64-pc-windows-msvc"
                | "x86_64-pc-windows-gnu"
        )
    }

    fn pair(&mut self, name: &str, value: &str) {
        self.pairs.push((name.to_string(), value.to_string()));
    }

    /// Adds one build-script `cargo::rustc-cfg` value (`name` or
    /// `name="value"`, the form Cargo passes to rustc as `--cfg`).
    /// The name becomes a known custom cfg name.
    pub fn add_rustc_cfg(&mut self, cfg: &str) {
        let cfg = cfg.trim();
        match cfg.split_once('=') {
            Some((name, value)) => {
                let value = value.trim().trim_matches('"');
                self.custom_names.insert(name.trim().to_string());
                self.custom_pair_start.get_or_insert(self.pairs.len());
                self.pair(name.trim(), value);
            }
            None => {
                self.custom_names.insert(cfg.to_string());
                if self.flags.insert(cfg.to_string()) {
                    self.custom_flags.insert(cfg.to_string());
                }
            }
        }
    }

    /// Records one build-script `cargo::rustc-check-cfg` value
    /// (`cfg(name, values(...))`, `cfg(name)`, or the old bare `name`)
    /// as a known custom cfg name, without setting it.
    pub fn add_check_cfg(&mut self, cfg: &str) {
        let cfg = cfg.trim();
        if cfg.split_once('(').is_some_and(|(head, _)| head.trim() == "cfg") {
            if let Ok(names) = check_cfg_names(cfg) {
                self.custom_names.extend(names);
            }
        } else {
            // Retain the old build-script record format. New Cargo manifest
            // declarations are validated with check_cfg_names before this call.
            let name = cfg.split('=').next().unwrap_or("").trim();
            if !name.is_empty() { self.custom_names.insert(name.to_string()); }
        }
    }

    /// True when `name` may appear in a predicate at all (section 1.6).
    pub fn knows(&self, name: &str) -> bool {
        KNOWN_CFG_NAMES.contains(&name)
            || self.custom_names.contains(name)
            || (self.allow_lust_dev && name == "lust_dev")
            || self.pairs.iter().any(|(n, _)| n == name)
    }

    /// The fingerprint input of this environment: every flag, pair, and
    /// feature in sorted order. Session keys and cache keys hash this text.
    pub fn describe(&self) -> String {
        let mut parts: Vec<String> = self.flags.iter().cloned().collect();
        let mut pairs: Vec<String> = self.pairs.iter().map(|(n, v)| format!("{n}={v}")).collect();
        pairs.sort();
        parts.extend(pairs);
        parts.extend(self.features.iter().map(|f| format!("feature={f}")));
        parts.extend(self.custom_names.iter().map(|f| format!("check-cfg={f}")));
        parts.join(" ")
    }

    /// Evaluates one predicate from its token trees. String tokens index
    /// `strings` (the owning AST's string table). A syntax shape outside
    /// the cfg grammar is L0001; an unknown name is L0520.
    pub fn eval(&self, trees: &[TokenTree], strings: &[String], interner: &Interner, span: Span) -> Result<bool, Diag> {
        let mut pos = 0;
        let v = self.pred(trees, &mut pos, strings, interner, span)?;
        if pos != trees.len() {
            return Err(Diag::new(
                "L0001",
                span,
                "malformed `#[cfg]` predicate",
                "Write `#[cfg(name)]`, `#[cfg(name = \"value\")]`, or `all`/`any`/`not` combinations.",
            ));
        }
        Ok(v)
    }

    fn pred(&self, trees: &[TokenTree], pos: &mut usize, strings: &[String], interner: &Interner, span: Span) -> Result<bool, Diag> {
        let name_of = |trees: &[TokenTree], at: usize| -> Option<(String, Span)> {
            match trees.get(at) {
                Some(TokenTree::Token(t)) => match t.tok {
                    Tok::Ident(s) => Some((interner.get(s).to_string(), t.span)),
                    Tok::True => Some(("true".into(), t.span)),
                    Tok::False => Some(("false".into(), t.span)),
                    _ => None,
                },
                _ => None,
            }
        };
        let Some((name, name_span)) = name_of(trees, *pos) else {
            return Err(Diag::new(
                "L0001",
                span,
                "malformed `#[cfg]` predicate",
                "Write `#[cfg(name)]`, `#[cfg(name = \"value\")]`, or `all`/`any`/`not` combinations.",
            ));
        };
        if let Some(d) = unstable_cfg(&name, name_span) {
            return Err(d);
        }
        *pos += 1;
        // Combinators: `all(...)`, `any(...)`, `not(...)`.
        if matches!(name.as_str(), "all" | "any" | "not") {
            let Some(TokenTree::Delim(_, inner, _)) = trees.get(*pos) else {
                return Err(Diag::new(
                    "L0001",
                    name_span,
                    format!("`{name}` in a cfg predicate needs `(...)`"),
                    "Write `all(a, b)`, `any(a, b)`, or `not(a)`.",
                ));
            };
            let inner = inner.clone();
            *pos += 1;
            let mut values = Vec::new();
            let mut ip = 0;
            while ip < inner.len() {
                values.push(self.pred(&inner, &mut ip, strings, interner, span)?);
                if matches!(inner.get(ip), Some(TokenTree::Token(t)) if t.tok == Tok::Comma) {
                    ip += 1;
                }
            }
            return Ok(match name.as_str() {
                "all" => values.iter().all(|v| *v),
                "any" => values.iter().any(|v| *v),
                _ => values.len() == 1 && !values[0],
            });
        }
        // `name = "value"` form.
        if matches!(trees.get(*pos), Some(TokenTree::Token(t)) if t.tok == Tok::Eq) {
            let value = match trees.get(*pos + 1) {
                Some(TokenTree::Token(t)) => match t.tok {
                    Tok::Str(i, _) => strings.get(i as usize).cloned().unwrap_or_default(),
                    Tok::Ident(s) => interner.get(s).to_string(),
                    _ => String::new(),
                },
                _ => String::new(),
            };
            *pos += 2;
            if name == "feature" {
                return Ok(self.features.contains(&value));
            }
            if !self.knows(&name) {
                return Err(unknown_cfg(&name, name_span));
            }
            return Ok(self.pairs.iter().any(|(n, v)| *n == name && *v == value));
        }
        // Bare `name` form.
        match name.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ if !self.knows(&name) => Err(unknown_cfg(&name, name_span)),
            _ => Ok(self.flags.contains(&name) || (self.allow_lust_dev && name == "lust_dev")),
        }
    }
}

/// Validate Cargo's explicit name declarations using the Rust lexer. The
/// expected values are lint input, never enabled configuration values.
/// Global name-check suppression is outside this bounded declaration form.
pub fn check_cfg_names(text: &str) -> Result<Vec<String>, String> {
    let interner = Interner::default();
    let lexed = crate::lexer::lex(text, crate::diag::FileId(0), 0, &interner)
        .map_err(|_| "invalid Rust tokens in check-cfg".to_string())?;
    let tokens: Vec<Tok> = lexed.tokens.iter().map(|t| t.tok).filter(|t| *t != Tok::Eof).collect();
    let name = |token: Option<&Tok>| match token { Some(Tok::Ident(s)) => Some(interner.get(*s)), _ => None };
    if name(tokens.first()) != Some("cfg") || tokens.get(1) != Some(&Tok::LParen) || tokens.last() != Some(&Tok::RParen) {
        return Err("check-cfg must be cfg(name, ..., values(...))".into());
    }
    let end = tokens.len() - 1;
    let mut at = 2;
    let mut names = Vec::new();
    while at < end {
        let Some(ident) = name(tokens.get(at)) else { return Err("expected a cfg name".into()); };
        at += 1;
        if tokens.get(at) == Some(&Tok::LParen) {
            if ident != "values" || names.is_empty() {
                return Err("check-cfg supports explicit cfg names and optional expected values".into());
            }
            at += 1;
            let mut any = false;
            let mut concrete = false;
            while at < end && tokens[at] != Tok::RParen {
                match tokens[at] {
                    Tok::Str(..) => {
                        concrete = true;
                        at += 1;
                    }
                    _ if matches!(name(tokens.get(at)), Some("none" | "any"))
                        && tokens.get(at + 1) == Some(&Tok::LParen)
                        && tokens.get(at + 2) == Some(&Tok::RParen) => {
                            if name(tokens.get(at)) == Some("any") {
                                if any { return Err("any() cannot be repeated in check-cfg values".into()); }
                                any = true;
                            } else {
                                concrete = true;
                            }
                            at += 3;
                        }
                    _ => return Err("expected a string, none() or any() in check-cfg values".into()),
                }
                if any && concrete { return Err("any() cannot be combined with other check-cfg values".into()); }
                if tokens.get(at) == Some(&Tok::Comma) { at += 1; }
                else if tokens.get(at) != Some(&Tok::RParen) { return Err("expected a comma in check-cfg values".into()); }
            }
            if tokens.get(at) != Some(&Tok::RParen) { return Err("unterminated check-cfg values".into()); }
            at += 1;
            if tokens.get(at) == Some(&Tok::Comma) { at += 1; }
            if at != end { return Err("expected values must be last in check-cfg".into()); }
            break;
        }
        names.push(ident.to_string());
        if at == end { break; }
        if tokens.get(at) != Some(&Tok::Comma) { return Err("expected a comma between cfg names".into()); }
        at += 1;
    }
    Ok(names)
}

/// The L0520 diagnostic for a cfg name lust does not know.
fn unknown_cfg(name: &str, span: Span) -> Diag {
    Diag::new(
        "L0520",
        span,
        format!("unknown cfg name `{name}`"),
        "Declare the cfg in build script output (`cargo::rustc-cfg`), or remove it. Lust knows the standard target, feature, and profile cfg names.",
    )
}

/// True when an attribute list stays in the program: every `#[cfg(...)]`
/// holds. `#[cfg_attr(...)]` does not gate the item; see `expand_cfg_attr`.
/// A `#[test]` item stays only in a test build, as in rustc: outside one
/// it is removed before name resolution, so a call to it does not resolve.
pub fn attrs_enabled(env: &CfgEnv, attrs: &[Attr], strings: &[String], interner: &Interner) -> Result<bool, Diag> {
    for a in attrs {
        if a.path.len() == 1 && interner.get(a.path[0].name) == "test" && !env.flags.contains("test") {
            return Ok(false);
        }
        if a.path.len() == 1
            && interner.get(a.path[0].name) == "cfg"
            && let AttrInput::List(_, trees) = &a.input
            && !env.eval(trees, strings, interner, a.span)?
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Rewrites `#[cfg_attr(pred, attr...)]` entries in an attribute list: a
/// true predicate splices the inner attributes in, a false one drops the
/// entry. Inner attributes are rebuilt from their token trees in the two
/// written forms (`name`, `name(...)`, `name = lit`); anything else keeps
/// its raw trees under a `List` input so later readers see the tokens.
pub fn expand_cfg_attr(env: &CfgEnv, attrs: &mut Vec<Attr>, strings: &[String], interner: &Interner) -> Result<(), Diag> {
    let mut out = Vec::with_capacity(attrs.len());
    for a in attrs.drain(..) {
        let is_cfg_attr = a.path.len() == 1 && interner.get(a.path[0].name) == "cfg_attr";
        if !is_cfg_attr {
            out.push(a);
            continue;
        }
        let AttrInput::List(_, trees) = &a.input else {
            out.push(a);
            continue;
        };
        // The predicate is the trees up to the first top-level comma.
        let comma = trees.iter().position(|t| matches!(t, TokenTree::Token(tok) if tok.tok == Tok::Comma));
        let Some(comma) = comma else {
            return Err(Diag::new("L0001", a.span, "malformed `#[cfg_attr]`", "Write `#[cfg_attr(predicate, attribute)]`."));
        };
        if !env.eval(&trees[..comma], strings, interner, a.span)? {
            continue;
        }
        // Split the rest into attributes at top-level commas. Each piece is
        // a path with an optional `(...)` or `= lit` tail.
        let mut start = comma + 1;
        while start < trees.len() {
            let mut end = start;
            while end < trees.len() && !matches!(&trees[end], TokenTree::Token(t) if t.tok == Tok::Comma) {
                end += 1;
            }
            if end > start {
                out.push(attr_from_trees(&trees[start..end], a.span, a.style));
            }
            start = end + 1;
        }
    }
    *attrs = out;
    Ok(())
}

/// One attribute rebuilt from cfg_attr token trees.
pub(crate) fn attr_from_trees(trees: &[TokenTree], span: Span, style: crate::ast::AttrStyle) -> Attr {
    let mut path = Vec::new();
    let mut i = 0;
    while let Some(TokenTree::Token(t)) = trees.get(i) {
        match t.tok {
            Tok::Ident(s) => {
                path.push(crate::ast::PathSeg::plain(s));
                i += 1;
                if matches!(trees.get(i), Some(TokenTree::Token(t2)) if t2.tok == Tok::PathSep) {
                    i += 1;
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
    let input = match trees.get(i) {
        Some(TokenTree::Delim(d, inner, _)) => AttrInput::List(*d, inner.clone()),
        Some(_) => AttrInput::List(crate::lexer::Delim::Bracket, trees[i..].to_vec()),
        None => AttrInput::None,
    };
    Attr { style, path, input, span }
}

/// Evaluates raw predicate trees from the scan outline (a `mod` declaration's
/// cfg), where the string table is the lexed file's.
pub fn outline_enabled(env: &CfgEnv, cfgs: &[Vec<TokenTree>], strings: &[String], interner: &Interner, span: Span) -> Result<bool, Diag> {
    for trees in cfgs {
        if !env.eval(trees, strings, interner, span)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The outline side of `expand_cfg_attr` for a `mod` declaration: false
/// when one of its `#[cfg_attr(pred, cfg(...))]` splices in a cfg that
/// is off, so a disabled file module is never loaded. Other spliced
/// attributes do not gate the declaration.
pub fn outline_cfg_attr_enabled(
    env: &CfgEnv,
    cfg_attrs: &[Vec<TokenTree>],
    strings: &[String],
    interner: &Interner,
    span: Span,
) -> Result<bool, Diag> {
    for trees in cfg_attrs {
        let comma = trees.iter().position(|t| matches!(t, TokenTree::Token(tok) if tok.tok == Tok::Comma));
        let Some(comma) = comma else {
            return Err(Diag::new("L0001", span, "malformed `#[cfg_attr]`", "Write `#[cfg_attr(predicate, attribute)]`."));
        };
        if !env.eval(&trees[..comma], strings, interner, span)? {
            continue;
        }
        // Each spliced attribute piece runs to the next top-level comma;
        // a piece that is `cfg(pred)` gates the declaration.
        let mut start = comma + 1;
        while start < trees.len() {
            let mut end = start;
            while end < trees.len() && !matches!(&trees[end], TokenTree::Token(t) if t.tok == Tok::Comma) {
                end += 1;
            }
            let piece = &trees[start..end];
            let is_cfg = matches!(piece.first(), Some(TokenTree::Token(t)) if matches!(t.tok, Tok::Ident(s) if interner.get(s) == "cfg"));
            if is_cfg
                && let Some(TokenTree::Delim(_, inner, _)) = piece.get(1)
                && !env.eval(inner, strings, interner, span)?
            {
                return Ok(false);
            }
            start = end + 1;
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diag::FileId;
    use crate::lexer;

    /// Lexes `#[cfg(<pred>)]` and evaluates the predicate trees.
    fn eval_pred(env: &CfgEnv, pred: &str) -> Result<bool, Diag> {
        let interner = Interner::default();
        let src = format!("#[cfg({pred})]\nfn f() {{}}");
        let lexed = lexer::lex(&src, FileId(0), 0, &interner).unwrap();
        let o = crate::scan::outline(&lexed, &interner);
        let _ = o;
        // Reuse the outline's cfg capture path through a mod decl instead.
        let src = format!("#[cfg({pred})]\nmod m;");
        let lexed = lexer::lex(&src, FileId(0), 0, &interner).unwrap();
        let o = crate::scan::outline(&lexed, &interner);
        outline_enabled(env, &o.mods[0].cfg, &lexed.strings, &interner, Span::default())
    }

    #[test]
    fn target_table_and_predicates() {
        let env = CfgEnv::new("x86_64-unknown-linux-gnu", &["beta".into()], false, ProfileFlags::DEV);
        assert!(eval_pred(&env, "unix").unwrap());
        assert!(!eval_pred(&env, "windows").unwrap());
        assert!(eval_pred(&env, "target_os = \"linux\"").unwrap());
        assert!(eval_pred(&env, "target_vendor = \"unknown\"").unwrap());
        assert!(eval_pred(&env, "target_pointer_width = \"64\"").unwrap());
        assert!(eval_pred(&env, "target_has_atomic = \"64\"").unwrap());
        assert!(eval_pred(&env, "feature = \"beta\"").unwrap());
        assert!(!eval_pred(&env, "feature = \"alpha\"").unwrap());
        assert!(eval_pred(&env, "all(unix, not(windows))").unwrap());
        assert!(eval_pred(&env, "any(windows, target_arch = \"x86_64\")").unwrap());
        assert!(eval_pred(&env, "debug_assertions").unwrap());
        assert!(!eval_pred(&env, "test").unwrap());
        let mac = CfgEnv::new("aarch64-apple-darwin", &[], true, ProfileFlags::DEV);
        assert!(eval_pred(&mac, "target_os = \"macos\"").unwrap());
        assert!(eval_pred(&mac, "target_vendor = \"apple\"").unwrap());
        assert!(eval_pred(&mac, "test").unwrap());
        let wasi = CfgEnv::new("wasm32-wasip1", &[], false, ProfileFlags::DEV);
        assert!(eval_pred(&wasi, "target_os = \"wasi\"").unwrap());
        assert!(!eval_pred(&wasi, "target_pointer_width = \"64\"").unwrap());
    }

    #[test]
    fn release_triples_match_rustc_cfg() {
        // Per `rustc --print cfg` 1.99: 128-bit atomics on aarch64 and on
        // x86_64 macOS/Windows, not on x86_64 Linux; musl is crt-static.
        let a64 = CfgEnv::new("aarch64-unknown-linux-musl", &[], false, ProfileFlags::DEV);
        assert!(eval_pred(&a64, "target_has_atomic = \"128\"").unwrap());
        assert!(eval_pred(&a64, "target_feature = \"crt-static\"").unwrap());
        assert!(eval_pred(&a64, "target_feature = \"neon\"").unwrap());
        assert!(eval_pred(&a64, "target_env = \"musl\"").unwrap());
        let x64 = CfgEnv::new("x86_64-unknown-linux-musl", &[], false, ProfileFlags::DEV);
        assert!(!eval_pred(&x64, "target_has_atomic = \"128\"").unwrap());
        assert!(eval_pred(&x64, "target_feature = \"crt-static\"").unwrap());
        let mac = CfgEnv::new("aarch64-apple-darwin", &[], false, ProfileFlags::DEV);
        assert!(eval_pred(&mac, "target_has_atomic = \"128\"").unwrap());
        assert!(eval_pred(&mac, "target_feature = \"lse\"").unwrap());
        assert!(eval_pred(&mac, "target_env = \"\"").unwrap());
        let win = CfgEnv::new("x86_64-pc-windows-msvc", &[], false, ProfileFlags::DEV);
        assert!(eval_pred(&win, "windows").unwrap());
        assert!(eval_pred(&win, "target_has_atomic = \"128\"").unwrap());
        assert!(eval_pred(&win, "target_feature = \"cmpxchg16b\"").unwrap());
        assert!(eval_pred(&win, "target_env = \"msvc\"").unwrap());
        assert!(CfgEnv::is_release_target("aarch64-pc-windows-msvc"));
        assert!(!CfgEnv::is_release_target("wasm32-wasip1"));
        assert!(!CfgEnv::is_release_target("riscv64gc-unknown-linux-gnu"));
    }

    #[test]
    fn unstable_cfg_name_is_l0523() {
        let env = CfgEnv::new("", &[], false, ProfileFlags::DEV);
        assert_eq!(eval_pred(&env, "overflow_checks").unwrap_err().code, "L0523");
        // Gated even where `any` would not need its value.
        assert_eq!(eval_pred(&env, "any(unix, target_thread_local)").unwrap_err().code, "L0523");
        assert!(eval_pred(&env, "target_has_atomic_primitive_alignment = \"8\"").unwrap());
    }

    #[test]
    fn unknown_cfg_name_is_l0520() {
        let env = CfgEnv::new("", &[], false, ProfileFlags::DEV);
        let e = eval_pred(&env, "frobnicate").unwrap_err();
        assert_eq!(e.code, "L0520");
        // `lust_dev` is unknown in user code, true in shipped crates.
        assert_eq!(eval_pred(&env, "lust_dev").unwrap_err().code, "L0520");
        let mut shipped = env.clone();
        shipped.allow_lust_dev = true;
        assert!(eval_pred(&shipped, "lust_dev").unwrap());
    }

    #[test]
    fn declared_names_are_known_but_not_enabled() {
        let mut env = CfgEnv::new("", &[], false, ProfileFlags::DEV);
        let before = env.describe();
        env.add_check_cfg("cfg(first_mode, second_mode, values(none(), \"wide\"))");
        assert!(!eval_pred(&env, "first_mode").unwrap());
        assert!(!eval_pred(&env, "second_mode = \"wide\"").unwrap());
        assert_ne!(before, env.describe());
        env.add_rustc_cfg("second_mode=\"wide\"");
        assert!(eval_pred(&env, "second_mode = \"wide\"").unwrap());
        for malformed in ["cfg(first_mode", "cfg(first_mode=\"x\")", "cfg(first_mode,,)", "cfg(first_mode, values(42))",
            "cfg(first_mode, values(any(), \"x\"))", "cfg(first_mode, values(none(), any()))", "cfg(first_mode, values(any(), any()))"] {
            assert!(check_cfg_names(malformed).is_err(), "{malformed}");
        }
    }

    #[test]
    fn dependency_cfg_is_per_crate() {
        let mut root = CfgEnv::new("x86_64-unknown-linux-gnu", &["root_feature".into()], true, ProfileFlags::DEV);
        let base = CfgEnv::new("x86_64-unknown-linux-gnu", &["dep_feature".into()], false, ProfileFlags::DEV);
        root.add_check_cfg("cfg(root_only, unix, target_arch)");
        root.add_rustc_cfg("root_mode");
        root.add_rustc_cfg("root_mode");
        root.add_rustc_cfg("root_value=\"wide\"");
        root.add_rustc_cfg("unix");
        root.add_rustc_cfg("target_arch=\"x86_64\"");
        root.allow_lust_dev = true;
        let mut dep = root.for_dependency(&["dep_feature".into()]);
        assert_eq!(dep, base, "only target/profile facts and the dependency's features survive");
        assert!(eval_pred(&root, "root_mode").unwrap());
        assert!(eval_pred(&root, "test").unwrap());
        assert_eq!(eval_pred(&dep, "root_only").unwrap_err().code, "L0520");
        assert_eq!(eval_pred(&dep, "root_mode").unwrap_err().code, "L0520");
        dep.add_check_cfg("cfg(dep_mode)");
        assert!(!eval_pred(&dep, "dep_mode").unwrap());
        dep.add_rustc_cfg("dep_mode");
        assert!(eval_pred(&dep, "dep_mode").unwrap());
        assert_eq!(eval_pred(&root, "dep_mode").unwrap_err().code, "L0520");
    }
}
