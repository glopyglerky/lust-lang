// Token-level scan for incremental compilation. Finds every function body (top level, in impl
// blocks, and nested in any block) without parsing, and hashes:
//   - each body's tokens (what can change without affecting any other function), and
//   - every token outside the bodies (the "environment": types, signatures, item order).
// Hashes ignore positions, so moving code around does not invalidate anything.
use crate::intern::{FxHasher, Sym};
use crate::lexer::{Lexed, Tok};
use std::hash::{Hash, Hasher};

pub struct FnScan {
    pub body_hash: u64,
    /// True for a fn nested inside another fn's body (a body item).
    pub nested: bool,
    /// Byte offset of the `fn` keyword.
    pub fn_offset: u32,
    /// Token range of the body, from `{` to `}` inclusive, within the chunk's tokens.
    pub body: std::ops::Range<usize>,
}

/// One top-level item, without function bodies.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ItemScan {
    pub hash: u64,
    /// The name, for a top-level `fn` item (whose hash is then its signature).
    pub fn_name: Option<Sym>,
}

pub struct ChunkScan {
    pub fns: Vec<FnScan>,
    /// Top-level items in order. Chunks always start at an item, so the concatenated list is
    /// the same however the file was split.
    pub items: Vec<ItemScan>,
    /// Identifiers used in each function body (parallel to `fns`), to find the bodies that
    /// mention a function whose signature changed.
    pub body_idents: Vec<Vec<Sym>>,
    /// The chunk ends exactly at a top-level item boundary: every item it started is complete.
    /// The driver only reuses a chunk in front of other text when this holds.
    pub ends_at_boundary: bool,
}

/// Combines item hashes in order into the environment hash.
pub fn env_hash<'a>(items: impl Iterator<Item = &'a ItemScan>) -> u64 {
    let mut h = FxHasher::default();
    for i in items {
        i.hash.hash(&mut h);
    }
    h.finish()
}

fn hash_tok(h: &mut FxHasher, t: &Tok, strings: &[String]) {
    std::mem::discriminant(t).hash(h);
    match t {
        Tok::Ident(s) | Tok::Reserved(s) => s.hash(h),
        Tok::Int(v, suf, _) => {
            v.hash(h);
            suf.map(|s| s as u8).hash(h);
        }
        Tok::Lifetime(s) => s.hash(h),
        Tok::Char(c, _) => c.hash(h),
        _ => {}
    }
    // Strings, including a literal's written text: `0x1` and `1` differ.
    for i in t.string_indices() {
        strings[i as usize].hash(h);
    }
}

/// A nested fn's body hash with the body hash of the top-level fn or method
/// that holds it folded in. The resolver path's Ast scan uses it too.
pub fn fold_outer(nested: u64, outer: u64) -> u64 {
    let mut h = FxHasher::default();
    "nested-in".hash(&mut h);
    (nested, outer).hash(&mut h);
    h.finish()
}

/// The scan of the fn item whose `fn` keyword is at token `k`: its body hash
/// (the caller folds in the enclosing body or item) and the identifiers its
/// body uses. A `;` before any body (a fn with no body) gives None.
fn nested_fn_scan(lexed: &Lexed, k: usize) -> Option<(FnScan, Vec<Sym>)> {
    let toks = &lexed.tokens;
    let mut j = k;
    let mut nest = 0usize;
    while j < toks.len() && toks[j].tok != Tok::LBrace {
        match toks[j].tok {
            Tok::LParen | Tok::LBracket => nest += 1,
            Tok::RParen | Tok::RBracket => nest = nest.saturating_sub(1),
            Tok::Semi if nest == 0 => return None,
            Tok::Eof => return None,
            _ => {}
        }
        j += 1;
    }
    let mut end = j;
    let mut d = 0usize;
    let mut body = FxHasher::default();
    let mut idents: Vec<Sym> = Vec::new();
    while end < toks.len() {
        match toks[end].tok {
            Tok::LBrace => d += 1,
            Tok::RBrace => d -= 1,
            Tok::Ident(s) => idents.push(s),
            Tok::Eof => return None,
            _ => {}
        }
        hash_tok(&mut body, &toks[end].tok, &lexed.strings);
        if d == 0 {
            break;
        }
        end += 1;
    }
    if d != 0 || end >= toks.len() {
        return None;
    }
    idents.sort_unstable();
    idents.dedup();
    Some((FnScan { body_hash: body.finish(), nested: true, fn_offset: toks[k].span.lo, body: j..end + 1 }, idents))
}

/// Scans one lexed chunk. Chunks start at top-level items, so brace depth starts at 0.
pub fn scan(lexed: &Lexed) -> ChunkScan {
    let toks = &lexed.tokens;
    let mut env = FxHasher::default();
    let mut items = Vec::new();
    let mut fns = Vec::new();
    let mut body_idents = Vec::new();
    // Nested fns ride after all top-level fns, in encounter order,
    // matching collect's LocalDefId order (W-CORELANG body items).
    let mut nested_fns: Vec<FnScan> = Vec::new();
    let mut nested_idents: Vec<Vec<Sym>> = Vec::new();
    let mut depth = 0usize;
    let mut impl_pending = false;
    let mut in_impl = false;
    let mut trait_pending = false;
    let mut in_trait = false;
    // Nested fns outside any counted fn body (in a trait default method
    // body, say), by index in `nested_fns`: their hashes fold in the
    // hash of the top-level item that holds them when it ends.
    let mut item_nested: Vec<usize> = Vec::new();
    let mut complete = true;
    let mut i = 0;
    while i < toks.len() {
        let t = &toks[i].tok;
        // A name must follow: a `fn(...)` pointer type in a tuple struct
        // field or an alias is not an item (it took the next item's
        // braces as its body, W-CHECK-G).
        let fn_here =
            *t == Tok::Fn && matches!(toks.get(i + 1).map(|t| t.tok), Some(Tok::Ident(_))) && (depth == 0 || (depth == 1 && in_impl));
        if fn_here {
            // The body is the first `{` at this depth after the signature.
            let mut j = i;
            // `-> impl Trait`: the body decides the hidden type callers
            // generate code for (check/opaque.rs), so it is part of the
            // environment, not a body-only edit.
            let mut arrow = false;
            let mut rpit = false;
            while j < toks.len() && toks[j].tok != Tok::LBrace && toks[j].tok != Tok::Eof {
                arrow |= toks[j].tok == Tok::Arrow;
                rpit |= arrow && toks[j].tok == Tok::Impl;
                hash_tok(&mut env, &toks[j].tok, &lexed.strings);
                j += 1;
            }
            if j >= toks.len() || toks[j].tok != Tok::LBrace {
                complete = false;
                break;
            }
            let mut k = j;
            let mut d = 0usize;
            let mut body = FxHasher::default();
            let mut idents = Vec::new();
            let nested_start = nested_fns.len();
            while k < toks.len() {
                // A nested fn item: record its body as its own scan. The
                // outer walk continues over its tokens, so deeper nesting
                // records too and the outer hash still covers everything.
                // A name must follow: a `fn(...)` pointer type is not an item.
                if toks[k].tok == Tok::Fn && k > j && matches!(toks.get(k + 1).map(|t| t.tok), Some(Tok::Ident(_)))
                    && let Some((f, nidents)) = nested_fn_scan(lexed, k)
                {
                    nested_fns.push(f);
                    nested_idents.push(nidents);
                }
                match toks[k].tok {
                    Tok::LBrace => d += 1,
                    Tok::RBrace => d -= 1,
                    Tok::Ident(s) => idents.push(s),
                    Tok::Eof => break,
                    _ => {}
                }
                hash_tok(&mut body, &toks[k].tok, &lexed.strings);
                if d == 0 {
                    break;
                }
                k += 1;
            }
            if d != 0 {
                complete = false;
            }
            let body_hash = body.finish();
            if rpit {
                body_hash.hash(&mut env);
            }
            // A nested fn reads the items of the blocks around it (a
            // sibling fn, const, or `use` that shadows a name), and no
            // recorded read covers those names. Its hash takes in the
            // whole enclosing body, so any edit there rechecks it.
            for f in &mut nested_fns[nested_start..] {
                f.body_hash = fold_outer(f.body_hash, body_hash);
            }
            fns.push(FnScan { body_hash, nested: false, fn_offset: toks[i].span.lo, body: j..k + 1 });
            idents.sort_unstable();
            idents.dedup();
            body_idents.push(idents);
            if depth == 0 {
                // A fn returning `impl Trait` changes callers' generated
                // code with its body: no fn-only shortcut, a full walk.
                let fn_name = match toks.get(i + 1).map(|t| t.tok) {
                    Some(Tok::Ident(s)) if !rpit => Some(s),
                    _ => None,
                };
                items.push(ItemScan { hash: std::mem::take(&mut env).finish(), fn_name });
            }
            i = k + 1;
            continue;
        }
        if *t == Tok::Eof {
            break;
        }
        // A fn item in a block outside the bodies above: in a trait
        // default method body (a direct trait member is the method
        // itself), or in a block in a type or an initializer. The parser
        // puts it in `stmt_items` like any nested fn, so it is scanned
        // as one; its tokens stay in the environment too.
        if *t == Tok::Fn
            && depth > 0
            && !(depth == 1 && in_trait)
            && matches!(toks.get(i + 1).map(|t| t.tok), Some(Tok::Ident(_)))
            && let Some((f, nidents)) = nested_fn_scan(lexed, i)
        {
            item_nested.push(nested_fns.len());
            nested_fns.push(f);
            nested_idents.push(nidents);
        }
        hash_tok(&mut env, t, &lexed.strings);
        let mut item_end = false;
        match t {
            Tok::Impl if depth == 0 => impl_pending = true,
            Tok::Trait if depth == 0 => trait_pending = true,
            Tok::LBrace => {
                if depth == 0 && impl_pending {
                    in_impl = true;
                    impl_pending = false;
                }
                if depth == 0 && trait_pending {
                    in_trait = true;
                    trait_pending = false;
                }
                depth += 1;
            }
            Tok::RBrace => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    in_impl = false;
                    in_trait = false;
                    item_end = true;
                }
            }
            Tok::Semi if depth == 0 => item_end = true,
            _ => {}
        }
        if item_end {
            let hash = std::mem::take(&mut env).finish();
            // Such a nested fn reads the names of the blocks around it,
            // and no recorded read covers them: any edit to its item
            // rechecks it (as `fold_outer` does for fn bodies).
            for k in item_nested.drain(..) {
                nested_fns[k].body_hash = fold_outer(nested_fns[k].body_hash, hash);
            }
            items.push(ItemScan { hash, fn_name: None });
        }
        i += 1;
    }
    // A chunk ends at an item boundary, so leftover tokens are only possible on a syntax error.
    if env.finish() != FxHasher::default().finish() {
        items.push(ItemScan { hash: env.finish(), fn_name: None });
        complete = false;
    }
    let ends_at_boundary = complete && depth == 0 && !in_impl && !impl_pending && !trait_pending;
    // Collect's order is the parser's `stmt_items` order, which is body
    // completion order: a fn nested inside a nested fn is pushed before
    // its enclosing fn. Encounter order here is the reverse for that
    // shape, so order the nested scans by body end to match. The driver
    // joins this list to LocalDefIds positionally; a swap makes a warm
    // edit reparse and reuse the wrong bodies (fuzz seeds 42 and 7 on
    // the first nested-items landing).
    let mut nested_pairs: Vec<(FnScan, Vec<Sym>)> = nested_fns.into_iter().zip(nested_idents).collect();
    nested_pairs.sort_by_key(|(f, _)| f.body.end);
    for (f, idents) in nested_pairs {
        fns.push(f);
        body_idents.push(idents);
    }
    ChunkScan { fns, items, body_idents, ends_at_boundary }
}

// ---------- Outline scan (docs/ARCH.md section 1.2) ----------
//
// Module discovery is a scan, not a full parse: the resolve worklist reads the
// outline of a file to decide which files to load and which macro names exist
// before any file is parsed. The outline records `mod` declarations (name,
// visibility, `#[path]`, raw `#[cfg]` tokens, inline or file), `use` header
// spans, and `macro_rules!` names, at module item level (the top level plus
// inline module bodies). It never looks inside fn bodies or impl blocks.
// Attribute contents stay raw tokens: cfg evaluation (src/resolve/cfg.rs) is
// the only reader, exactly as with the AST attribute record.

/// Visibility of an outline item, as written. The restricted path itself is
/// read from the AST later; the outline only needs the three-way split.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutlineVis {
    Private,
    Pub,
    Restricted,
}

/// One `mod` declaration in a file outline.
#[allow(dead_code)] // Fields are read by the resolve worklist as it lands.
#[derive(Clone, Debug)]
pub struct ModDecl {
    pub name: Sym,
    pub vis: OutlineVis,
    /// The value of a `#[path = "..."]` attribute, when written.
    pub path_attr: Option<String>,
    /// The span of a `#[path]` attribute whose input is not
    /// `= "..."`, when one is written. The resolver reports it as
    /// L0524: module files load from this outline, so the parser's
    /// own input check never gets to judge this case first.
    pub path_bad: Option<crate::diag::Span>,
    /// The token trees inside each `#[cfg(...)]` on the declaration, raw.
    pub cfg: Vec<Vec<crate::lexer::TokenTree>>,
    /// The token trees inside each `#[cfg_attr(...)]` on the declaration,
    /// raw: a true predicate can splice in a `cfg` that disables the mod.
    pub cfg_attr: Vec<Vec<crate::lexer::TokenTree>>,
    /// True for `mod m { ... }`, false for `mod m;`.
    pub inline: bool,
    /// True when a `#[macro_use]` attribute is present.
    pub macro_use: bool,
    /// The inline module path that contains this declaration (empty at the
    /// top level of the file). One segment per enclosing inline `mod`.
    pub container: Vec<Sym>,
    pub span: crate::diag::Span,
}

/// One `macro_rules!` definition name in a file outline.
#[allow(dead_code)] // Fields are read by the resolve worklist as it lands.
#[derive(Clone, Debug)]
pub struct MacroDecl {
    pub name: Sym,
    /// True when a `#[macro_export]` attribute is present.
    pub exported: bool,
    /// True when `#[macro_export]` also carries `local_inner_macros`.
    pub local_inner: bool,
    /// (inline container depth, token index of `macro_rules`).
    pub container: (usize, usize),
    /// The inline module name path that contains this definition (empty
    /// at the top level of the file), as in `ModDecl::container`.
    pub container_path: Vec<Sym>,
    pub span: crate::diag::Span,
}

/// The outline of one lexed file: module edges, import headers, macro names.
#[derive(Clone, Debug, Default)]
pub struct Outline {
    pub mods: Vec<ModDecl>,
    /// Spans of `use` declarations at module item level, for hashing and for
    /// the worklist's quick "has imports" test. The trees themselves come
    /// from the AST.
    pub uses: Vec<crate::diag::Span>,
    pub macros: Vec<MacroDecl>,
    /// True when the file starts with an inner `#![...]` attribute list that
    /// names `macro_use` (all modules see textual macros) — recorded for the
    /// macro scope walk; other inner attributes are the parser's business.
    pub inner_macro_use: bool,
}

/// Attributes pending in front of an outline item, reduced to what the
/// outline records. String values come from the lexed string table.
#[derive(Default)]
struct PendingAttrs {
    path_attr: Option<String>,
    path_bad: Option<crate::diag::Span>,
    cfg: Vec<Vec<crate::lexer::TokenTree>>,
    cfg_attr: Vec<Vec<crate::lexer::TokenTree>>,
    macro_use: bool,
    macro_export: bool,
    local_inner: bool,
}

/// Builds token trees from a flat token slice (same nesting rule as the
/// parser's attribute storage). Used for `#[cfg(...)]` contents, here
/// and in the resolver's pre-expansion cfg check.
pub fn outline_trees(toks: &[crate::lexer::Token]) -> Vec<crate::lexer::TokenTree> {
    use crate::lexer::{Delim, TokenTree};
    fn delim_of(t: Tok) -> Option<Delim> {
        match t {
            Tok::LParen => Some(Delim::Paren),
            Tok::LBracket => Some(Delim::Bracket),
            Tok::LBrace => Some(Delim::Brace),
            _ => None,
        }
    }
    fn closes(open: Delim, t: Tok) -> bool {
        matches!((open, t), (Delim::Paren, Tok::RParen) | (Delim::Bracket, Tok::RBracket) | (Delim::Brace, Tok::RBrace))
    }
    fn build(toks: &[crate::lexer::Token], pos: &mut usize, close: Option<Delim>, depth: u32) -> Vec<TokenTree> {
        let mut out = Vec::new();
        while let Some(t) = toks.get(*pos) {
            if close.is_some_and(|d| closes(d, t.tok)) {
                *pos += 1;
                break;
            }
            *pos += 1;
            if let Some(d) = delim_of(t.tok).filter(|_| depth < 256) {
                let inner = build(toks, pos, Some(d), depth + 1);
                out.push(TokenTree::Delim(d, inner, t.span));
            } else {
                out.push(TokenTree::Token(*t));
            }
        }
        out
    }
    let mut pos = 0;
    build(toks, &mut pos, None, 0)
}

/// Scans one whole lexed file into an outline. The file must be lexed from
/// its start (brace depth begins at 0), unlike chunk scans.
pub fn outline(lexed: &Lexed, interner: &crate::intern::Interner) -> Outline {
    let toks = &lexed.tokens;
    // A symbol compare per identifier token, not a text read.
    let macro_rules = interner.intern("macro_rules");
    let mut out = Outline::default();
    // Brace stack: Some(name) for an inline module body, None for any other
    // brace. Items are outline-visible exactly when every open brace is an
    // inline module body.
    let mut braces: Vec<Option<Sym>> = Vec::new();
    let mut pending = PendingAttrs::default();
    let mut pending_vis = OutlineVis::Private;
    // (name span, container depth) of a `mod name` whose `{` is still coming.
    let mut mod_brace_for: Option<Sym> = None;
    let mut i = 0;
    let at_item_level = |braces: &[Option<Sym>]| braces.iter().all(|b| b.is_some());
    let container_of = |braces: &[Option<Sym>]| braces.iter().filter_map(|b| *b).collect::<Vec<Sym>>();
    while i < toks.len() {
        let t = toks[i].tok;
        match t {
            Tok::Eof => break,
            Tok::LBrace => {
                braces.push(if at_item_level(&braces) { mod_brace_for.take() } else { None });
                i += 1;
            }
            Tok::RBrace => {
                braces.pop();
                pending = PendingAttrs::default();
                pending_vis = OutlineVis::Private;
                mod_brace_for = None;
                i += 1;
            }
            _ if !at_item_level(&braces) => {
                i += 1;
            }
            Tok::Pound => {
                // One attribute: `#` `!`? `[` ... `]`. Read the name, then
                // reduce the forms the outline records.
                let mut j = i + 1;
                let inner = toks.get(j).map(|t| t.tok) == Some(Tok::Bang);
                if inner {
                    j += 1;
                }
                if toks.get(j).map(|t| t.tok) != Some(Tok::LBracket) {
                    i += 1;
                    continue;
                }
                j += 1;
                let body_start = j;
                let mut depth = 1usize;
                while j < toks.len() && depth > 0 {
                    match toks[j].tok {
                        Tok::LBracket => depth += 1,
                        Tok::RBracket => depth -= 1,
                        Tok::Eof => break,
                        _ => {}
                    }
                    j += 1;
                }
                let body = &toks[body_start..j.saturating_sub(1).max(body_start)];
                let name = body.first().and_then(|t| match t.tok {
                    Tok::Ident(s) => Some(interner.get(s).to_string()),
                    _ => None,
                });
                if inner && braces.is_empty() && name.as_deref() == Some("macro_use") {
                    out.inner_macro_use = true;
                }
                match name.as_deref() {
                    Some("path") => {
                        // The one form rustc accepts: `path = "file.rs"`.
                        match body {
                            [_, eq, s] if eq.tok == Tok::Eq => match s.tok {
                                Tok::Str(x, _) => pending.path_attr = lexed.strings.get(x as usize).cloned(),
                                _ => pending.path_bad = Some(toks[i].span),
                            },
                            _ => pending.path_bad = Some(toks[i].span),
                        }
                    }
                    Some("cfg") | Some("cfg_attr") => {
                        // `cfg(pred)` / `cfg_attr(pred, attr)`: the trees
                        // inside the first paren group.
                        if let Some(open) = body.iter().position(|t| t.tok == Tok::LParen) {
                            let mut d = 0usize;
                            let mut end = body.len();
                            for (k, t) in body.iter().enumerate().skip(open) {
                                match t.tok {
                                    Tok::LParen => d += 1,
                                    Tok::RParen => {
                                        d -= 1;
                                        if d == 0 {
                                            end = k;
                                            break;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            let trees = outline_trees(&body[open + 1..end]);
                            if name.as_deref() == Some("cfg_attr") {
                                pending.cfg_attr.push(trees);
                            } else {
                                pending.cfg.push(trees);
                            }
                        }
                    }
                    Some("macro_use") => pending.macro_use = true,
                    Some("macro_export") => {
                        pending.macro_export = true;
                        pending.local_inner =
                            body.iter().any(|t| matches!(t.tok, Tok::Ident(s) if interner.get(s) == "local_inner_macros"));
                    }
                    _ => {}
                }
                i = j;
            }
            Tok::Pub => {
                pending_vis = if toks.get(i + 1).map(|t| t.tok) == Some(Tok::LParen) { OutlineVis::Restricted } else { OutlineVis::Pub };
                i += 1;
            }
            Tok::Mod => {
                let span = toks[i].span;
                let name = match toks.get(i + 1).map(|t| t.tok) {
                    Some(Tok::Ident(s)) => Some(s),
                    _ => None,
                };
                if let Some(name) = name {
                    let after = toks.get(i + 2).map(|t| t.tok);
                    let inline = after == Some(Tok::LBrace);
                    if inline {
                        mod_brace_for = Some(name);
                    }
                    out.mods.push(ModDecl {
                        name,
                        vis: pending_vis,
                        path_attr: pending.path_attr.take(),
                        path_bad: pending.path_bad.take(),
                        cfg: std::mem::take(&mut pending.cfg),
                        cfg_attr: std::mem::take(&mut pending.cfg_attr),
                        inline,
                        macro_use: pending.macro_use,
                        container: container_of(&braces),
                        span,
                    });
                    i += 2;
                } else {
                    i += 1;
                }
                pending = PendingAttrs::default();
                pending_vis = OutlineVis::Private;
            }
            // `use<..>` is a precise-capturing bound, not a `use` item.
            Tok::Use if toks.get(i + 1).map(|t| t.tok) != Some(Tok::Lt) => {
                out.uses.push(toks[i].span);
                pending = PendingAttrs::default();
                pending_vis = OutlineVis::Private;
                i += 1;
            }
            Tok::Ident(s) if s == macro_rules && toks.get(i + 1).map(|t| t.tok) == Some(Tok::Bang) => {
                if let Some(Tok::Ident(name)) = toks.get(i + 2).map(|t| t.tok) {
                    let container = container_of(&braces);
                    out.macros.push(MacroDecl {
                        name,
                        exported: pending.macro_export,
                        local_inner: pending.local_inner,
                        container: (container.len(), i),
                        container_path: container,
                        span: toks[i].span,
                    });
                }
                pending = PendingAttrs::default();
                pending_vis = OutlineVis::Private;
                i += 1;
            }
            Tok::Semi => {
                pending = PendingAttrs::default();
                pending_vis = OutlineVis::Private;
                mod_brace_for = None;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod outline_tests {
    use super::*;
    use crate::diag::FileId;
    use crate::intern::Interner;
    use crate::lexer;

    fn scan_outline(src: &str) -> (Outline, Interner) {
        let interner = Interner::default();
        let lexed = lexer::lex(src, FileId(0), 0, &interner).unwrap();
        (outline(&lexed, &interner), interner)
    }

    #[test]
    fn outline_records_mods_uses_and_macros() {
        let src = "\
pub mod a;
#[cfg(feature = \"x\")]
mod b;
#[path = \"else/where.rs\"]
pub(crate) mod c;
mod outer {
    pub mod inner;
    fn hidden() { let _ = 1; }
}
use a::Thing;
#[macro_use]
mod withmacros;
#[macro_export]
macro_rules! make_it { () => { 1 } }
fn main() { println!(\"{}\", 1); }
";
        let (o, interner) = scan_outline(src);
        let names: Vec<&str> = o.mods.iter().map(|m| interner.get(m.name)).collect();
        assert_eq!(names, ["a", "b", "c", "outer", "inner", "withmacros"]);
        assert_eq!(o.mods[0].vis, OutlineVis::Pub);
        assert!(!o.mods[1].cfg.is_empty(), "cfg tokens recorded");
        assert_eq!(o.mods[2].path_attr.as_deref(), Some("else/where.rs"));
        assert_eq!(o.mods[2].vis, OutlineVis::Restricted);
        assert!(o.mods[3].inline);
        assert_eq!(o.mods[4].container.len(), 1, "inner lives in outer");
        assert!(o.mods[5].macro_use);
        assert_eq!(o.uses.len(), 1);
        assert_eq!(o.macros.len(), 1);
        assert_eq!(interner.get(o.macros[0].name), "make_it");
        assert!(o.macros[0].exported);
    }

    #[test]
    fn outline_ignores_fn_bodies_and_impls() {
        let src = "fn f() { let mod_name = 1; println!(\"{}\", mod_name); }\nstruct S;\nimpl S { fn m(&self) {} }\n";
        let (o, _) = scan_outline(src);
        assert!(o.mods.is_empty() && o.uses.is_empty() && o.macros.is_empty());
    }

    // A nested fn's hash takes in its enclosing top-level fn body: a
    // sibling item can shadow a name the nested fn calls, and no recorded
    // read covers it. An edit in another fn keeps the hash.
    #[test]
    fn nested_fn_hash_covers_its_enclosing_body() {
        let interner = Interner::default();
        let nested_hash = |src: &str| {
            let lexed = lexer::lex(src, FileId(0), 0, &interner).unwrap();
            scan(&lexed).fns.iter().find(|f| f.nested).map(|f| f.body_hash).unwrap()
        };
        let src = "fn h() -> i32 { 1 }\nfn f() -> i32 {\n    fn g() -> i32 { h() }\n    g()\n}\n";
        let a = nested_hash(src);
        assert_ne!(a, nested_hash(&src.replace("    g()\n", "    fn h() -> i32 { 2 }\n    g()\n")));
        assert_eq!(a, nested_hash(&src.replace("{ 1 }", "{ 3 }")));
    }

    // The driver's fn table joins scan order to collect's LocalDefId order
    // positionally, so the scan's nested fns must be in the same order as
    // the parser's `stmt_items` fns, also for a fn nested inside a nested fn,
    // in a trait default body, or in a block in a type.
    #[test]
    fn scan_nested_fn_order_matches_parser() {
        let src = "\
trait Tr {
    fn r(&self) -> i32;
    fn d(&self) -> [i32; 2] { fn h() -> i32 { fn h() -> i32 { 1 } h() } fn m(x: i32) -> i32 { x } [h(), m(2)] }
}
struct S { a: [u8; { fn z() -> usize { 3 } 3 }] }
fn main() {
    fn a() -> i32 {
        fn b() -> i32 { 10 }
        b() + 1
    }
    fn c() -> i32 { 100 }
    println!(\"{}\", a() + c());
}
";
        let interner = Interner::default();
        let lexed = lexer::lex(src, FileId(0), 0, &interner).unwrap();
        let known = crate::intern::Known::new(&interner);
        let ast = crate::parser::parse(&lexed, &interner, &known).unwrap();
        let parser_order: Vec<&str> = ast
            .stmt_items
            .iter()
            .filter_map(|it| match &it.kind {
                crate::ast::ItemKind::Fn(d) => Some(interner.get(d.name)),
                _ => None,
            })
            .collect();
        let scanned = scan(&lexed);
        let scan_order: Vec<&str> = scanned
            .fns
            .iter()
            .filter(|f| f.nested)
            .map(|f| {
                let pos = lexed.tokens.iter().position(|t| t.span.lo == f.fn_offset).unwrap();
                match lexed.tokens[pos + 1].tok {
                    Tok::Ident(s) => interner.get(s),
                    _ => "?",
                }
            })
            .collect();
        assert_eq!(scan_order, parser_order, "scan nested order must match parser stmt_items order");
        // Trait methods are not counted fns; `main` is the only top-level one.
        assert_eq!(scanned.fns.iter().filter(|f| !f.nested).count(), 1);
    }
}
