//! Macro scope by text for crates with dependencies (sigedit P4.3).
//!
//! A body patch expands the pure builtin macros of an edited body alone
//! (driver/patch.rs `expand_pure`). That matches a full resolve only when
//! each such name means the builtin in the full resolve too. In a crate
//! with dependencies, a macro can take a builtin's name in the local
//! crate only through its own source:
//! - a `macro_rules!` of that name (or of a `$x` name);
//! - a `macro_rules!` a macro expansion defines (one inside a macro body or
//!   a macro call's input): its scope follows rustc's rules for
//!   macro-expanded macros (E0659), which this scan does not model;
//! - `#[macro_use] extern crate dep;` (any `extern crate` but the std
//!   libraries refuses here);
//! - a `use` whose path passes through a dependency that is not clean
//!   (below), or through a name a `use` of such a path bound (`use dep as
//!   d;`, then `crate::d::..`), that brings a builtin's name or a glob;
//! - a `use` from any path that renames to a builtin's name, or a
//!   one-segment `use name;` of one (the re-export of a `macro_rules!`).
//!
//! Any other `use` passes on only bindings the crate already has, so a
//! macro under a builtin's name needs one of the forms above first.
//! Path-qualified macro calls (`dep::m!`) never patch.
//!
//! A clean dependency (`clean`) compiles from source in the session and
//! its own files pass the same scan: then no path through it reaches a
//! macro under a builtin's name, and its paths taint nothing (ts_goport
//! re-exports its path dependencies with `pub use goport_util::core::*;`).
//!
//! The scan reads tokens, not the merged Ast (the resolver drops `use`
//! items when it flattens): `use` trees inside `macro_rules!` bodies and
//! cfg-disabled code count too, and `$x` segments count as dependency
//! paths. Anything the scan cannot read refuses.

use crate::diag::FileId;
use crate::intern::{FxMap, Interner, Sym};
use crate::lexer::{self, Tok, Token};

/// True when no macro but the builtin can take a builtin macro's name in
/// the crate whose files are `texts`: none of the forms in the file
/// header. `deps` are the names the crate reaches its dependencies by
/// (renames included; not `std`, `core`, `alloc`); `clean(d)` says
/// dependency `d` is clean.
pub(crate) fn dep_macros_shut(texts: &[&str], interner: &Interner, deps: &[Sym], clean: &dyn Fn(Sym) -> bool) -> bool {
    let builtin = |s: Sym| {
        let name = interner.get(s);
        crate::expand::builtin::is_passthrough(name) || crate::expand::builtin::is_token_builtin(name)
    };
    let per_file = crate::pool::par_map(texts, |text| {
        let lexed = lexer::lex(text, FileId(0), 0, interner).ok()?;
        let mut out = Vec::new();
        file_uses(&lexed.tokens, interner, &builtin, &mut out).then_some(out)
    });
    let mut uses: Vec<Flat> = Vec::new();
    for f in per_file {
        match f {
            Some(u) => uses.extend(u),
            None => return false,
        }
    }
    // A path through a dependency that is not clean: one with a tainted
    // name in any segment, a `$x` metavariable first, or a `::name` path to
    // any crate but the std libraries and clean dependencies.
    let std_lib = |s: Sym| matches!(interner.get(s), "std" | "core" | "alloc");
    let dollar = interner.intern("$");
    let from_dep = |u: &Flat, tainted: &FxMap<Sym, ()>| match u.path.first() {
        None => true,
        Some(&f) => {
            f == dollar
                || (u.leading_colon && !std_lib(f) && !(deps.contains(&f) && clean(f)))
                || u.path.iter().any(|s| tainted.contains_key(s))
        }
    };
    // Names a use of such a path binds, to a fixpoint.
    let mut tainted: FxMap<Sym, ()> = deps.iter().filter(|&&d| !clean(d)).map(|&d| (d, ())).collect();
    loop {
        let before = tainted.len();
        for u in &uses {
            if from_dep(u, &tainted)
                && let Some(b) = u.bound
            {
                tainted.insert(b, ());
            }
        }
        if tainted.len() == before {
            break;
        }
    }
    uses.iter().all(|u| {
        let renames_or_reexports_builtin = (u.renamed || u.path.len() == 1) && u.bound.is_some_and(builtin);
        !renames_or_reexports_builtin && !(from_dep(u, &tainted) && (u.glob || u.bound.is_some_and(builtin)))
    })
}

/// One flattened use path: `a::b::c as d` binds `d`, `a::b::{self}`
/// binds `b`, `a::T as _` binds nothing, and a glob brings every name
/// (`glob`).
struct Flat {
    leading_colon: bool,
    path: Vec<Sym>,
    bound: Option<Sym>,
    renamed: bool,
    glob: bool,
}

/// Adds every `use` tree of one file's tokens to `out`. False on an
/// `extern crate` of a crate but the std libraries, a `macro_rules!` of a
/// builtin's name (`builtin`) or of a `$x` name, a `macro_rules!` inside a
/// macro body or a macro call's input, or a tree the scan cannot read.
fn file_uses(toks: &[Token], interner: &Interner, builtin: &dyn Fn(Sym) -> bool, out: &mut Vec<Flat>) -> bool {
    let at = |i: usize| toks.get(i).map(|t| &t.tok);
    let std_lib = |t: Option<&Tok>| matches!(t, Some(Tok::Ident(s)) if matches!(interner.get(*s), "std" | "core" | "alloc"));
    let is_macro_rules = |t: Option<&Tok>| matches!(t, Some(Tok::Ident(s)) if interner.get(*s) == "macro_rules");
    // Open groups, each true when it is a macro body (`macro_rules! m {`)
    // or a macro call's input (`m!(`, not `#![`).
    let mut groups: Vec<bool> = Vec::new();
    for i in 0..toks.len() {
        match &toks[i].tok {
            Tok::LParen | Tok::LBracket | Tok::LBrace => {
                let back = |k: usize| i.checked_sub(k).and_then(|j| toks.get(j)).map(|t| &t.tok);
                let call = back(1) == Some(&Tok::Bang) && back(2) != Some(&Tok::Pound);
                let body = matches!(back(1), Some(Tok::Ident(_))) && back(2) == Some(&Tok::Bang) && is_macro_rules(back(3));
                groups.push(call || body);
            }
            Tok::RParen | Tok::RBracket | Tok::RBrace => {
                groups.pop();
            }
            Tok::Extern if at(i + 1) == Some(&Tok::Crate) && !std_lib(at(i + 2)) => return false,
            Tok::Ident(m) if at(i + 1) == Some(&Tok::Bang) && interner.get(*m) == "macro_rules" => match at(i + 2) {
                Some(Tok::Ident(name)) if !builtin(*name) && !groups.contains(&true) => {}
                _ => return false,
            },
            // `use<'a, T>` is precise capturing, not an import.
            Tok::Use if at(i + 1) != Some(&Tok::Lt) => {
                let leading = at(i + 1) == Some(&Tok::PathSep);
                let start = i + 1 + leading as usize;
                if tree(toks, start, interner, leading, &mut Vec::new(), out).is_none() {
                    return false;
                }
            }
            _ => {}
        }
    }
    true
}

/// Reads one use tree from `toks[i]` under `prefix`, adding its paths to
/// `out`. Returns the index after it, or None for a shape it cannot read.
fn tree(toks: &[Token], mut i: usize, interner: &Interner, leading: bool, prefix: &mut Vec<Sym>, out: &mut Vec<Flat>) -> Option<usize> {
    let depth = prefix.len();
    let tok = |i: usize| toks.get(i).map(|t| &t.tok);
    let flat = |path: &Vec<Sym>, bound, renamed, glob| Flat { leading_colon: leading, path: path.clone(), bound, renamed, glob };
    loop {
        match tok(i)? {
            Tok::Star => {
                out.push(flat(prefix, None, false, true));
                i += 1;
                break;
            }
            Tok::LBrace => {
                i += 1;
                while tok(i)? != &Tok::RBrace {
                    i = tree(toks, i, interner, leading, prefix, out)?;
                    match tok(i)? {
                        Tok::Comma => i += 1,
                        Tok::RBrace => {}
                        _ => return None,
                    }
                }
                i += 1;
                break;
            }
            t => {
                let seg = match t {
                    Tok::Ident(s) => *s,
                    Tok::Crate => interner.intern("crate"),
                    Tok::Super => interner.intern("super"),
                    Tok::SelfLower => interner.intern("self"),
                    // `$crate` is the defining crate: in a local macro, the
                    // local crate. Any other `$x` may name anything.
                    Tok::Dollar if tok(i + 1) == Some(&Tok::Crate) => {
                        i += 1;
                        interner.intern("crate")
                    }
                    Tok::Dollar if matches!(tok(i + 1), Some(Tok::Ident(_))) => {
                        i += 1;
                        interner.intern("$")
                    }
                    _ => return None,
                };
                prefix.push(seg);
                i += 1;
                if tok(i) == Some(&Tok::PathSep) {
                    i += 1;
                    continue;
                }
                let last = prefix.iter().rev().find(|&&s| interner.get(s) != "self").copied();
                if tok(i) == Some(&Tok::As) {
                    match tok(i + 1)? {
                        Tok::Ident(r) => out.push(flat(prefix, Some(*r), true, false)),
                        Tok::Underscore => out.push(flat(prefix, None, false, false)),
                        _ => return None,
                    }
                    i += 2;
                } else {
                    out.push(flat(prefix, last, false, false));
                }
                break;
            }
        }
    }
    prefix.truncate(depth);
    Some(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `dep_macros_shut` for one file, with dependency `bumpalo`.
    fn shut(src: &str) -> bool {
        let interner = Interner::default();
        dep_macros_shut(&[src], &interner, &[interner.intern("bumpalo")], &|_| false)
    }

    #[test]
    fn local_paths_and_dep_types_stay_shut() {
        assert!(shut("use crate::format; use super::*; use self::m::*; mod m {} fn main() {}"));
        assert!(shut("use bumpalo::Bump; use bumpalo::collections::Vec as BVec; use bumpalo::Tr as _; fn main() {}"));
        assert!(shut("use std::fmt::Write; use std::vec; use ::std::io::*; pub(crate) use {crate::a::{b, c::*}}; fn main() {}"));
        assert!(shut("fn f() -> impl Sized + use<> {} macro_rules! m { () => { use $crate::x::*; } } fn main() {}"));
    }

    #[test]
    fn dep_imports_of_builtin_names_open() {
        assert!(!shut("use bumpalo::vec; fn main() {}"));
        assert!(!shut("use bumpalo::{Bump, format}; fn main() {}"));
        assert!(!shut("use bumpalo::*; fn main() {}"));
        assert!(!shut("use bumpalo::collections::*; fn main() {}"));
        assert!(!shut("use ::other::matches; fn main() {}"));
        assert!(!shut("use {bumpalo::write}; fn main() {}"));
        assert!(!shut("mod m { pub use bumpalo::vec; } fn main() {}"));
        assert!(!shut("fn f() { use bumpalo::vec; } fn main() {}"));
        assert!(!shut("macro_rules! m { ($d:ident) => { use $d::*; } } fn main() {}"));
    }

    #[test]
    fn aliases_of_deps_and_builtin_renames_open() {
        assert!(!shut("use bumpalo as b; use b::vec; fn main() {}"));
        assert!(!shut("use bumpalo::{self as b}; use b::*; fn main() {}"));
        assert!(!shut("use bumpalo::collections; use collections::*; fn main() {}"));
        assert!(!shut("use crate::m::thing as matches; mod m {} fn main() {}"));
        assert!(!shut("#[macro_use] extern crate bumpalo; fn main() {}"));
        assert!(shut("extern crate alloc; fn main() {}"));
    }

    #[test]
    fn builtin_named_macros_open() {
        assert!(!shut("macro_rules! matches { () => {} } fn main() {}"));
        assert!(!shut("macro_rules! dbg { () => {} } fn main() {}"));
        assert!(!shut("macro_rules! m { ($n:ident) => { macro_rules! $n { () => {} } } } fn main() {}"));
        assert!(!shut("mod m { pub(crate) use matches; } fn main() {}"));
        assert!(shut("macro_rules! twice { ($e:expr) => { $e * 2 } } pub(crate) use twice; fn main() {}"));
    }

    #[test]
    fn macro_expanded_macros_open() {
        assert!(!shut("macro_rules! d { () => { macro_rules! mym { () => {} } } } d!(); fn main() {}"));
        assert!(!shut("macro_rules! id { ($($t:tt)*) => { $($t)* } } id! { macro_rules! mym { () => {} } } fn main() {}"));
        assert!(shut("#![allow(unused)] macro_rules! a { () => { 1 } } fn main() { let _ = vec![a!()]; }"));
    }

    #[test]
    fn clean_deps_taint_nothing() {
        let interner = Interner::default();
        let (b, u) = (interner.intern("bumpalo"), interner.intern("util"));
        let shut = |src: &str| dep_macros_shut(&[src], &interner, &[b, u], &|d| d == u);
        assert!(shut("pub use util::core::*; use util::*; use ::util::vec; use util as v; use v::x::*; fn main() {}"));
        assert!(!shut("pub use bumpalo::core::*; fn main() {}"));
        // An alias of a dependency that is not clean taints every path
        // through it, not only paths that start at it.
        assert!(!shut("use bumpalo as g; use crate::g::*; fn main() {}"));
        assert!(!shut("pub use bumpalo::sub; mod m { use crate::sub::vec; } fn main() {}"));
        assert!(shut("pub use bumpalo::sub; mod m { use crate::sub::Thing; } fn main() {}"));
    }
}
