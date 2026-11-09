// Input and target checks for the built-in attributes the parser accepts
// and otherwise ignores (`inline`, `cold`, `must_use`, `deprecated`,
// `repr` targets, `test`, `ignore`, `should_panic`, and the lint levels).
// rustc rejects a malformed
// input or a wrong target for these, so ignoring the attribute would be a
// false accept. A lint raised to deny or forbid is L0531: rustc would
// reject code that triggers the lint, and lust does not check lints.
use super::PResult;
use crate::ast::{Attr, AttrInput};
use crate::diag::Diag;
use crate::intern::Interner;
use crate::lexer::{Delim, Tok, TokenTree};

/// The code for a built-in attribute with a malformed input or on a
/// target rustc does not permit.
const C_ATTR_USE: &str = "L0524";

/// Lint tools whose lints rustc itself never raises: a level on one of
/// them cannot reject a program under rustc.
const LINT_TOOLS: &[&str] = &["clippy", "rustdoc"];

/// Checks the input shape of one single-segment attribute named `name`.
/// Called for every attribute the parser keeps; names it does not judge
/// pass.
pub(super) fn check_input(name: &str, attr: &Attr, interner: &Interner) -> PResult<()> {
    let ok = match name {
        "inline" => match &attr.input {
            AttrInput::None => true,
            AttrInput::List(Delim::Paren, t) => {
                matches!(t.as_slice(), [TokenTree::Token(x)] if matches!(x.tok, Tok::Ident(s) if matches!(interner.get(s), "always" | "never")))
            }
            _ => false,
        },
        "cold" | "test" | "non_exhaustive" | "global_allocator" => matches!(attr.input, AttrInput::None),
        "must_use" => matches!(&attr.input, AttrInput::None) || eq_str(&attr.input),
        // `ignore` and `ignore = "reason"` are the only forms (probed).
        "ignore" => matches!(attr.input, AttrInput::None) || eq_str(&attr.input),
        // `should_panic`, `should_panic = "text"`, and
        // `should_panic(expected = "text")` are the only forms.
        "should_panic" => match &attr.input {
            AttrInput::None => true,
            AttrInput::List(Delim::Paren, t) => {
                matches!(comma_pieces(t).as_slice(), [only] if key_str(only, interner).as_deref() == Some("expected"))
            }
            other => eq_str(other),
        },
        // `#[path = "..."]` is the only form rustc accepts.
        "path" => eq_str(&attr.input),
        "deprecated" => match &attr.input {
            AttrInput::None => true,
            AttrInput::List(Delim::Paren, t) => deprecated_list(t, interner),
            other => eq_str(other),
        },
        "allow" | "expect" | "warn" | "deny" | "forbid" => return lint_list(name, attr, interner),
        _ => true,
    };
    if ok { Ok(()) } else { Err(malformed(name, attr)) }
}

/// Target flags for `check_target`: what kind of thing the attributes
/// sit on. rustc hard-errors on some misplacements and only warns
/// (future-incompatible) on others; a warning-only placement is
/// accepted here, as rustc accepts it today.
pub(super) const T_FN: u8 = 1;
pub(super) const T_ADT: u8 = 2;
pub(super) const T_VARIANT: u8 = 4;
pub(super) const T_FIELD: u8 = 8;
pub(super) const T_PARAM: u8 = 16;
pub(super) const T_UNION: u8 = 32;
pub(super) const T_STATIC: u8 = 64;
/// A trait impl block, or an item inside one (with `T_FN` for a method).
pub(super) const T_TRAIT_IMPL: u8 = 128;

/// Rejects a built-in attribute on a target rustc rejects it on.
/// `what` names the target in rustc's words ("structs", "constants").
/// The rules: `inline` takes functions (a field only warns), `cold`
/// warns everywhere except function parameters, which take only lint
/// and cfg attributes, `repr` takes structs, enums, and unions, and
/// `non_exhaustive` takes structs, enums, and variants (a field only
/// warns).
pub(super) fn check_target(attrs: &[Attr], what: &str, flags: u8, interner: &Interner) -> PResult<()> {
    for a in attrs {
        if a.path.len() != 1 {
            continue;
        }
        let name = interner.get(a.path[0].name);
        let bad = match name {
            "inline" => flags & (T_FN | T_FIELD) == 0,
            "cold" => flags & T_PARAM != 0,
            // The test harness attributes only warn on wrong item
            // targets, but a function parameter takes only lint and
            // cfg attributes, so there rustc rejects them (probed).
            "ignore" | "should_panic" => flags & T_PARAM != 0,
            "repr" => flags & (T_ADT | T_UNION) == 0,
            "non_exhaustive" => flags & (T_ADT | T_VARIANT | T_FIELD) == 0,
            // rustc: allocators must be statics.
            "global_allocator" => flags & T_STATIC == 0,
            // rustc: functions only; a field or a match arm only warns,
            // and a statement is judged by its kind (`stmt_track_caller`).
            "track_caller" => flags & (T_FN | T_FIELD) == 0 && what != "match arms" && what != "statements",
            // rustc's `useless_deprecated` (a trait impl method or block)
            // and its target check (an extern crate, `what` "extern crates").
            "deprecated" => (flags & T_TRAIT_IMPL != 0 && (flags & T_FN != 0 || what == "trait impl blocks")) || what == "extern crates" || flags & T_PARAM != 0,
            _ => flags & T_PARAM != 0 && !matches!(name, "allow" | "cfg" | "cfg_attr" | "deny" | "expect" | "forbid" | "warn"),
        };
        if bad && flags & T_PARAM != 0 && !matches!(name, "inline" | "cold" | "ignore" | "should_panic" | "repr" | "non_exhaustive" | "global_allocator") {
            return Err(Diag::new(
                C_ATTR_USE,
                a.span,
                "allow, cfg, cfg_attr, deny, expect, forbid, and warn are the only allowed built-in attributes in function parameters",
                format!("Remove the `#[{name}]` attribute."),
            ));
        }
        if bad {
            let what = if name == "deprecated" && flags & T_FN != 0 { "trait methods in impl blocks" } else { what };
            return Err(Diag::new(
                C_ATTR_USE,
                a.span,
                format!("the `{name}` attribute cannot be used on {what}"),
                format!("Remove the `#[{name}]` attribute."),
            ));
        }
    }
    Ok(())
}

/// The malformed-input error for attribute `name`.
fn malformed(name: &str, attr: &Attr) -> Diag {
    Diag::new(C_ATTR_USE, attr.span, format!("malformed `{name}` attribute input"), "Write the attribute in one of the forms Rust documents for it.")
}

/// True for an `= "text"` input (a plain or raw string literal).
fn eq_str(input: &AttrInput) -> bool {
    matches!(input, AttrInput::List(Delim::Bracket, t)
        if matches!(t.as_slice(), [TokenTree::Token(e), TokenTree::Token(s)] if e.tok == Tok::Eq && matches!(s.tok, Tok::Str(..))))
}

/// Splits trees at top-level commas; a trailing comma adds no piece.
fn comma_pieces(trees: &[TokenTree]) -> Vec<&[TokenTree]> {
    let mut out: Vec<&[TokenTree]> =
        trees.split(|t| matches!(t, TokenTree::Token(x) if x.tok == Tok::Comma)).collect();
    if out.last().is_some_and(|p| p.is_empty()) {
        out.pop();
    }
    out
}

/// The name of a `key = "text"` piece, or None for any other shape.
fn key_str(piece: &[TokenTree], interner: &Interner) -> Option<String> {
    match piece {
        [TokenTree::Token(k), TokenTree::Token(e), TokenTree::Token(v)] if e.tok == Tok::Eq && matches!(v.tok, Tok::Str(..)) => match k.tok {
            Tok::Ident(s) => Some(interner.get(s).to_string()),
            _ => None,
        },
        _ => None,
    }
}

/// `deprecated(since = "..", note = "..")`: each key once, nothing else.
fn deprecated_list(trees: &[TokenTree], interner: &Interner) -> bool {
    let mut seen: Vec<String> = Vec::new();
    for piece in comma_pieces(trees) {
        match key_str(piece, interner) {
            Some(k) if (k == "since" || k == "note") && !seen.contains(&k) => seen.push(k),
            _ => return false,
        }
    }
    true
}

/// A lint level list: lint paths (`name` or `tool::name`), then an
/// optional final `reason = "text"`. A lint lust does not check at
/// deny or forbid is L0531.
fn lint_list(level: &str, attr: &Attr, interner: &Interner) -> PResult<()> {
    let AttrInput::List(Delim::Paren, trees) = &attr.input else {
        return Err(malformed(level, attr));
    };
    let pieces = comma_pieces(trees);
    for (i, piece) in pieces.iter().enumerate() {
        if key_str(piece, interner).as_deref() == Some("reason") && i + 1 == pieces.len() {
            continue;
        }
        let Some(path) = lint_path(piece, interner) else {
            return Err(Diag::new(C_ATTR_USE, attr.span, "malformed lint attribute input", "Name each lint as `name` or `tool::name`."));
        };
        let tool = path.split_once("::").map(|(t, _)| t);
        if let Some(t) = tool
            && !LINT_TOOLS.contains(&t)
        {
            return Err(Diag::new(
                C_ATTR_USE,
                attr.span,
                format!("unknown lint tool `{t}` in `{path}`"),
                "Lust knows the `clippy` and `rustdoc` lint tools.",
            ));
        }
        // `unsafe_code` needs no lint pass: lust rejects `unsafe`
        // itself, so denying it can never hide a rustc reject.
        if matches!(level, "deny" | "forbid") && tool.is_none() && path != "unsafe_code" {
            return Err(Diag::new(
                "L0531",
                attr.span,
                format!("lint `{path}` is set to {level}, but lust does not check it"),
                format!("Remove the `#[{level}]` or lower it to warn, or build this crate with Cargo."),
            ));
        }
    }
    Ok(())
}

/// The text of a lint path piece (`a` or `a::b::c`), or None.
fn lint_path(piece: &[TokenTree], interner: &Interner) -> Option<String> {
    let mut out = String::new();
    for (i, t) in piece.iter().enumerate() {
        let TokenTree::Token(t) = t else { return None };
        match (i % 2, t.tok) {
            (0, Tok::Ident(s)) => out.push_str(interner.get(s)),
            (1, Tok::PathSep) => out.push_str("::"),
            _ => return None,
        }
    }
    (!out.is_empty() && !out.ends_with("::")).then_some(out)
}

