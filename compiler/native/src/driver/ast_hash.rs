// Structural AST scan for the resolver path: per-item hashes (everything
// except fn bodies) and per-fn body hashes plus body identifier sets, in
// the same shapes `scan::scan` produces from tokens for the single-file
// path. The worklist's merged Ast is post-expansion, post-cfg-strip, and
// path-rewritten, so hashing it covers macro-definition edits and cfg
// strip effects that a source-text body hash cannot see. Spans are never
// hashed (positions do not change meaning, and an edit above a body must
// not rehash it), except the hygiene context of a name that resolves
// among locals and labels, numbered per walk (`Walker::ctxt`); string-table
// indices hash by content, because indices
// shift when other files change. Matches are exhaustive with no wildcard
// arm: a new Ast variant fails to compile here instead of silently
// hashing as "unchanged", which would be a false-reuse bug.
use crate::ast::*;
use crate::intern::{FxHasher, Sym};
use crate::lexer::{Tok, TokenTree};
use crate::scan;
use std::hash::{Hash, Hasher};

/// One scan of a merged crate Ast: item scans parallel to `ast.items`,
/// and per-fn (body hash, body idents) in fn-location order (top-level
/// fns and impl methods in item order, as `collect` numbers them).
pub struct AstScans {
    pub items: Vec<scan::ItemScan>,
    pub bodies: Vec<(u64, Vec<Sym>)>,
}

/// One item's scan, its fns' body scans, and the body items (`stmt_items`
/// indexes) at any depth inside it.
pub(crate) type ItemScans = (scan::ItemScan, Vec<(u64, Vec<Sym>)>, Vec<u32>);

/// The fns of one item in scan order (a fn item's fn, an impl's methods).
fn item_fns<'a>(item: &'a Item) -> Vec<&'a FnDef> {
    match &item.kind {
        ItemKind::Fn(f) => vec![f],
        ItemKind::Impl(imp) => imp.fns.iter().collect(),
        _ => Vec::new(),
    }
}

/// One item's scan from its head walk and its fns' body hashes: a fn
/// returning `impl Trait` folds its body hash in, because the body
/// decides the hidden type callers generate code for
/// (check/opaque.rs), so it is part of the item, not a body-only edit
/// (as in `scan::scan`).
fn item_scan(ast: &Ast, item: &Item, body_hashes: &[u64], inner: Option<&mut Vec<u32>>) -> scan::ItemScan {
    let mut w = Walker { ast, idents: None, inner, quals: None, muted: 0, calls_only: false, bare: Vec::new(), ctxts: Vec::new() };
    let mut h = FxHasher::default();
    w.item(&mut h, item, false);
    // The type items in the item's bodies (structs, enums, traits,
    // aliases, impls) are program state, not body state: collect
    // records them like module items, and a warm request keeps that
    // record while the item scan stands. So they fold in here, and an
    // edit to one is an item edit (tf-bodyitems). Without the body
    // walk's list (`scan_item_reuse`), the spans inside the item find
    // them; per-item hashes fold sorted, so the order does not count.
    let types: Vec<u32> = match w.inner.as_deref() {
        Some(list) => list.to_vec(),
        None => (0..ast.stmt_items.len() as u32).filter(|&i| span_holds(item.span, ast.stmt_items[i as usize].span)).collect(),
    };
    let mut type_hashes: Vec<u64> = types
        .into_iter()
        .map(|i| &ast.stmt_items[i as usize])
        .filter(|it| is_body_type_item(&it.kind))
        .map(|it| item_hash(ast, it))
        .collect();
    if !type_hashes.is_empty() {
        type_hashes.sort_unstable();
        "body-type-items".hash(&mut h);
        type_hashes.hash(&mut h);
    }
    let rpit = |f: &FnDef| f.ret.is_some_and(|t| matches!(ast.ty(t), TyExpr::ImplTrait { .. }));
    let fn_name = match &item.kind {
        ItemKind::Fn(f) if !rpit(f) => Some(f.name),
        _ => None,
    };
    for (f, b) in item_fns(item).iter().zip(body_hashes) {
        if rpit(f) {
            b.hash(&mut h);
        }
    }
    scan::ItemScan { hash: h.finish(), fn_name }
}

/// True for an item kind inside a fn body that collect records as
/// program state (`item_scan`).
fn is_body_type_item(kind: &ItemKind) -> bool {
    matches!(kind, ItemKind::Struct(_) | ItemKind::Enum(_) | ItemKind::Union(_) | ItemKind::Trait(_) | ItemKind::Type(_) | ItemKind::Impl(_))
}

/// The index of the item that holds item `k` of `ast.items`, when `k` is
/// an impl that flatten hoisted out of a fn body (`ImplDef::in_body`): the
/// last item before it that is not one, if its span holds it (flatten puts
/// each body impl right after its holder).
pub(crate) fn body_impl_holder(ast: &Ast, k: usize) -> Option<usize> {
    let hoisted = |it: &Item| matches!(&it.kind, ItemKind::Impl(imp) if imp.in_body);
    if !hoisted(ast.items.get(k)?) {
        return None;
    }
    let h = (0..k).rev().find(|&h| !hoisted(&ast.items[h]))?;
    span_holds(ast.items[h].span, ast.items[k].span).then_some(h)
}

/// True when span `inner` lies inside span `outer` (same file).
fn span_holds(outer: crate::diag::Span, inner: crate::diag::Span) -> bool {
    outer.file == inner.file && outer.lo <= inner.lo && inner.hi <= outer.hi
}

/// One item's scan and its fns' body scans (see `ItemScans`).
pub(crate) fn scan_item(ast: &Ast, item: &Item) -> ItemScans {
    let mut inner = Vec::new();
    let bodies: Vec<(u64, Vec<Sym>)> = item_fns(item).iter().map(|f| body_scan(ast, f, Some(&mut inner))).collect();
    let hashes: Vec<u64> = bodies.iter().map(|b| b.0).collect();
    // The head walk also records body items (`inner`), as the scan the
    // worklist caches per file does.
    let scan = item_scan(ast, item, &hashes, Some(&mut inner));
    (scan, bodies, inner)
}

/// One fn's body scan (hash, idents, and the body items inside it), for
/// the body patch's one-method rescan of an impl.
pub(crate) fn scan_body(ast: &Ast, f: &FnDef) -> (u64, Vec<Sym>, Vec<u32>) {
    let mut inner = Vec::new();
    let (hash, idents) = body_scan(ast, f, Some(&mut inner));
    (hash, idents, inner)
}

/// One item's scan reusing stored body hashes for all but one fn (the
/// body patch's rescan after a body edit in an impl): `body_hashes` is
/// the item's fn body hashes in scan order, with the edited fn's entry
/// already replaced. The same scan `scan_item` computes when the stored
/// hashes are the bodies' current hashes.
pub(crate) fn scan_item_reuse(ast: &Ast, item: &Item, body_hashes: &[u64]) -> scan::ItemScan {
    item_scan(ast, item, body_hashes, None)
}

fn assemble(ast: &Ast, per_item: Vec<ItemScans>) -> AstScans {
    let mut items = Vec::with_capacity(per_item.len());
    let mut bodies = Vec::new();
    // The content hash (header and bodies) of the top-level item that
    // holds each body item: `scan::fold_outer` takes it into a nested
    // fn's body hash, as the token scan does, so any edit around a
    // nested fn rechecks it (it reads the names of the blocks around it).
    // The methods of a body impl that flatten hoisted to the module items
    // (`ImplDef::in_body`) fold it in too: they read the block items, and a
    // block struct edit is an edit of the holding item only.
    let mut outer = vec![0u64; ast.stmt_items.len()];
    // The content of the last item that is not a hoisted body impl: it
    // holds the body impls that follow it (`body_impl_holder`).
    let mut holder: Option<(usize, u64)> = None;
    for (k, (scan, mut bs, inner)) in per_item.into_iter().enumerate() {
        let content = item_content(&scan, &bs);
        for i in inner {
            if let Some(o) = outer.get_mut(i as usize) {
                *o = content;
            }
        }
        match holder {
            Some((h, c)) if body_impl_holder(ast, k) == Some(h) => bs.iter_mut().for_each(|b| b.0 = scan::fold_outer(b.0, c)),
            _ => holder = Some((k, content)),
        }
        items.push(scan);
        bodies.extend(bs);
    }
    // Nested fns follow the top-level fns, in stmt item order (collect's order).
    for (i, item) in ast.stmt_items.iter().enumerate() {
        if let ItemKind::Fn(f) = &item.kind {
            let (hash, idents) = body_scan(ast, f, None);
            bodies.push((scan::fold_outer(hash, outer[i]), idents));
        }
    }
    AstScans { items, bodies }
}

/// Scans one merged Ast. Item and body hashing are read-only walks,
/// so they run in parallel across items (one pass produces each
/// item's scan and its fns' body scans). See the module docs.
pub(crate) fn scan_ast(ast: &Ast) -> AstScans {
    assemble(ast, super::compile::par_map(&ast.items, |item| scan_item(ast, item)))
}

/// Scans one per-file Ast chunk serially. The worklist scans each
/// flattened chunk on its own (in parallel across files) and caches
/// the result per file; chunk scans concatenate to the merged scan
/// because string indices hash by content and spans never hash.
pub(crate) fn scan_ast_seq(ast: &Ast) -> AstScans {
    assemble(ast, ast.items.iter().map(|item| scan_item(ast, item)).collect())
}

/// The structural hash of one item without fn bodies (the item-scan
/// hash of `scan_ast`). A trait item hashes its default bodies too.
pub(crate) fn item_hash(ast: &Ast, item: &Item) -> u64 {
    let mut w = Walker { ast, idents: None, inner: None, quals: None, muted: 0, calls_only: false, bare: Vec::new(), ctxts: Vec::new() };
    let mut h = FxHasher::default();
    w.item(&mut h, item, false);
    h.finish()
}

/// The hash of one fn's head: name, qualifiers, generics, self and
/// parameter bindings (names, patterns, types), return type, and
/// where clause. The driver stores it per slot: a body is checked
/// against its own head, so a head change always rechecks the fn
/// itself, even when its body hash and caller-visible `Sig`
/// fingerprint (which excludes parameter names) both stand.
pub(crate) fn fn_head_hash(ast: &Ast, f: &FnDef) -> u64 {
    let mut w = Walker { ast, idents: None, inner: None, quals: None, muted: 0, calls_only: false, bare: Vec::new(), ctxts: Vec::new() };
    let mut h = FxHasher::default();
    "ast-fn-head-v1".hash(&mut h);
    w.fn_head(&mut h, f, false);
    h.finish()
}

/// The body hash and sorted identifier set of one fn. `inner`, when set,
/// gains the body items inside it.
/// The content hash (header and bodies) of one top-level item, from its
/// `scan_item` output: a nested fn's body hash folds it in.
pub(crate) fn item_content(scan: &scan::ItemScan, bodies: &[(u64, Vec<Sym>)]) -> u64 {
    let mut h = FxHasher::default();
    scan.hash.hash(&mut h);
    bodies.iter().for_each(|b| b.0.hash(&mut h));
    h.finish()
}

/// A const fn body as an evaluator root (`ConstReach`, `Walker::quals`):
/// its identifiers but bare names (`Walker::bare`), the last segments of
/// qualified paths and method call names, and those paths' (qualifier,
/// name) pairs ((name, name) for a method call).
pub(crate) fn body_quals(ast: &Ast, f: &FnDef) -> (Vec<Sym>, Vec<(Sym, Sym)>) {
    let (mut idents, mut quals) = (Vec::new(), Vec::new());
    if let Some(b) = f.body {
        Walker { ast, idents: Some(&mut idents), inner: None, quals: Some(&mut quals), muted: 0, calls_only: false, bare: Vec::new(), ctxts: Vec::new() }.block(&mut FxHasher::default(), b);
    }
    (idents, quals)
}

pub(crate) fn body_scan(ast: &Ast, f: &FnDef, inner: Option<&mut Vec<u32>>) -> (u64, Vec<Sym>) {
    let mut idents = Vec::new();
    let mut w = Walker { ast, idents: Some(&mut idents), inner, quals: None, muted: 0, calls_only: false, bare: Vec::new(), ctxts: Vec::new() };
    let mut h = FxHasher::default();
    "ast-body-v1".hash(&mut h);
    match f.body {
        Some(b) => w.block(&mut h, b),
        None => 0u8.hash(&mut h),
    }
    let hash = h.finish();
    let mut idents = idents;
    idents.sort_unstable();
    idents.dedup();
    (hash, idents)
}

/// What `const_idents` finds: identifiers in the places other than item
/// values (array lengths, const generic arguments, `const` blocks,
/// pattern bounds, discriminants, const param defaults), and per const,
/// static, or associated const item, its name and the identifiers of its
/// value. Item values also give `quals`: (qualifier, name) for each path
/// of two or more segments (`LazyLock::new`), (name, name) for each method
/// call (`.get(..)`), and `value_unqual`: their
/// identifiers in any other place but bare names (`Walker::bare`). Identifiers inside a closure body count
/// only inside a `const` block there (`Walker::quals`).
#[derive(Default)]
pub(crate) struct ConstRoots {
    pub scalar: Vec<Sym>,
    pub values: Vec<(Option<Sym>, Vec<Sym>)>,
    pub quals: Vec<(Sym, Sym)>,
    pub value_unqual: Vec<Sym>,
    /// The method call names in the places other than item values (not in
    /// `scalar`).
    pub scalar_calls: Vec<Sym>,
    /// The bare names (`Walker::bare`) there (not in `scalar` either).
    pub scalar_bare: Vec<Sym>,
}

/// The const expressions an item holds outside fn bodies, for
/// `const_idents`: const and static values, discriminants, associated
/// const values, and const param defaults, in nested modules too. Types
/// sit in the type arena, which `const_idents` scans on its own.
fn item_consts(ast: &Ast, item: &Item, self_sym: Sym, out: &mut ConstRoots) {
    // Every identifier of `e` into `idents`: first the ones that are not
    // bare or the names of qualified paths (their count is returned), then
    // those (their pairs into `quals` too).
    fn walk_quals(ast: &Ast, e: ExprId, idents: &mut Vec<Sym>, quals: &mut Vec<(Sym, Sym)>) -> usize {
        let (from, start) = (quals.len(), idents.len());
        let mut w = Walker { ast, idents: Some(idents), inner: None, quals: Some(quals), muted: 0, calls_only: false, bare: Vec::new(), ctxts: Vec::new() };
        w.expr(&mut FxHasher::default(), e);
        let bare = std::mem::take(&mut w.bare);
        let unqual = idents.len() - start;
        idents.extend(bare);
        idents.extend(quals[from..].iter().map(|q| q.1));
        unqual
    }
    fn walk(ast: &Ast, e: ExprId, idents: &mut Vec<Sym>) {
        walk_quals(ast, e, idents, &mut Vec::new());
    }
    fn generics(ast: &Ast, g: &Generics, out: &mut ConstRoots) {
        for p in &g.params {
            if let GenericParam::Const { default: Some(e), .. } = p {
                walk(ast, *e, &mut out.scalar);
            }
        }
    }
    fn value(ast: &Ast, name: Option<Sym>, e: Option<ExprId>, out: &mut ConstRoots) {
        if let Some(e) = e {
            let mut idents = Vec::new();
            let unqual = walk_quals(ast, e, &mut idents, &mut out.quals);
            out.value_unqual.extend_from_slice(&idents[..unqual]);
            out.values.push((name, idents));
        }
    }
    match &item.kind {
        ItemKind::Const(c) => value(ast, c.name, c.value, out),
        ItemKind::Static(st) => value(ast, Some(st.name), st.value, out),
        ItemKind::Fn(f) => generics(ast, &f.generics, out),
        ItemKind::Struct(st) => generics(ast, &st.generics, out),
        ItemKind::Type(a) => generics(ast, &a.generics, out),
        ItemKind::Union(u) => generics(ast, &u.generics, out),
        ItemKind::Enum(en) => {
            generics(ast, &en.generics, out);
            for e in en.variants.iter().filter_map(|v| v.discriminant) {
                walk(ast, e, &mut out.scalar);
            }
        }
        ItemKind::Impl(imp) => {
            generics(ast, &imp.generics, out);
            imp.fns.iter().for_each(|f| generics(ast, &f.generics, out));
            // `Self::f` in an impl's const names the self type's `f`.
            let from = out.quals.len();
            imp.consts.iter().for_each(|c| value(ast, Some(c.name), c.default, out));
            if let TyExpr::Path(segs) = ast.ty(imp.self_ty)
                && let Some(last) = segs.last()
            {
                out.quals[from..].iter_mut().filter(|q| q.0 == self_sym).for_each(|q| q.0 = last.name);
            }
        }
        ItemKind::Trait(t) => {
            generics(ast, &t.generics, out);
            t.fns.iter().for_each(|f| generics(ast, &f.generics, out));
            t.consts.iter().for_each(|c| value(ast, Some(c.name), c.default, out));
        }
        ItemKind::Mod(m) => m.body.iter().flatten().for_each(|it| item_consts(ast, it, self_sym, out)),
        ItemKind::ExternBlock(b) => {
            for it in &b.items {
                if let ForeignItem::Static(st) = it {
                    value(ast, Some(st.name), st.value, out);
                }
            }
        }
        ItemKind::Use(_) | ItemKind::MacroRules(_) | ItemKind::ExternCrate(_) | ItemKind::MacroCall(_) => {}
    }
}

/// Identifiers a const evaluation may read (`driver/const_reach.rs`):
/// those inside every array length, const generic argument, `const`
/// block, and pattern range bound of the arena nodes from `from`
/// (expression, type, and pattern arena lengths) to the end. With
/// `items`, also those of every item and body item outside fn bodies
/// (`item_consts`). `bodies` (sorted expression ranges of fn bodies),
/// when given, keeps a `const` block outside them out of `scalar`: it
/// sits in an item value, which `values` covers.
pub(crate) fn const_idents(ast: &Ast, from: (usize, usize, usize), items: bool, bodies: Option<&[(u32, u32)]>, self_sym: Sym) -> ConstRoots {
    let mut out = ConstRoots::default();
    if items {
        for it in ast.items.iter().chain(&ast.stmt_items) {
            item_consts(ast, it, self_sym, &mut out);
        }
    }
    let mut calls = Vec::new();
    let mut w = Walker { ast, idents: Some(&mut out.scalar), inner: None, quals: Some(&mut calls), muted: 0, calls_only: true, bare: Vec::new(), ctxts: Vec::new() };
    let mut h = FxHasher::default();
    // The const arguments of one path segment's (or turbofish's) angle
    // arguments.
    fn args(w: &mut Walker, h: &mut FxHasher, args: &[GenericArg]) {
        for a in args {
            if let GenericArg::Const(e) | GenericArg::Binding { kind: BindingKind::EqConst(e), .. } = a {
                w.expr(h, *e);
            }
        }
    }
    fn segs(w: &mut Walker, h: &mut FxHasher, segs: &[PathSeg]) {
        for s in segs {
            if let GenericArgs::Angle(a) = &s.args {
                args(w, h, a);
            }
        }
    }
    let in_body = |i: usize| {
        bodies.is_none_or(|r| {
            let k = r.partition_point(|&(lo, _)| lo as usize <= i);
            k > 0 && i < r[k - 1].1 as usize
        })
    };
    for (i, e) in ast.exprs.iter().enumerate().skip(from.0) {
        match e {
            Expr::ArrayRepeat { len, .. } => w.expr(&mut h, *len),
            Expr::ConstBlock(b) if in_body(i) => w.block(&mut h, *b),
            Expr::Path(p) | Expr::StructLit { path: p, .. } => segs(&mut w, &mut h, p),
            Expr::QPath { trait_path, segs: p, .. } => {
                segs(&mut w, &mut h, trait_path);
                segs(&mut w, &mut h, p);
            }
            Expr::MethodCall { generic_args, .. } => args(&mut w, &mut h, generic_args),
            _ => {}
        }
    }
    for t in ast.tys.get(from.1..).unwrap_or(&[]) {
        match t {
            TyExpr::Array { len, .. } => w.expr(&mut h, *len),
            TyExpr::Path(p) => segs(&mut w, &mut h, p),
            TyExpr::QPath { trait_path, segs: p, .. } => {
                segs(&mut w, &mut h, trait_path);
                segs(&mut w, &mut h, p);
            }
            TyExpr::Dyn { bounds, .. } | TyExpr::ImplTrait { bounds, .. } => {
                for b in bounds {
                    if let TraitBound::Trait { trait_ref, .. } = b {
                        segs(&mut w, &mut h, &trait_ref.path);
                    }
                }
            }
            _ => {}
        }
    }
    for p in ast.pats.get(from.2..).unwrap_or(&[]) {
        match p {
            Pat::Path(p) | Pat::TupleStruct { path: p, .. } | Pat::Struct { path: p, .. } => segs(&mut w, &mut h, p),
            Pat::QPath { trait_path, segs: p, .. } => {
                segs(&mut w, &mut h, trait_path);
                segs(&mut w, &mut h, p);
            }
            Pat::Range { lo, hi, .. } => {
                for e in lo.iter().chain(hi) {
                    w.expr(&mut h, *e);
                }
            }
            Pat::Lit(e) => w.expr(&mut h, *e),
            _ => {}
        }
    }
    out.scalar_bare = std::mem::take(&mut w.bare);
    out.scalar_calls = calls.into_iter().map(|(_, n)| n).collect();
    out
}

struct Walker<'a> {
    ast: &'a Ast,
    /// When set (body scans), every hashed identifier is also recorded.
    idents: Option<&'a mut Vec<Sym>>,
    /// When set (top-level item scans), every body item walked is recorded.
    inner: Option<&'a mut Vec<u32>>,
    /// Set for const evaluation roots (`item_consts`): a path of two or
    /// more segments gives (qualifier, name) here in place of its last
    /// name in `idents`, a method call `.name(..)` gives (name, name),
    /// and identifiers inside a closure body are not
    /// recorded, except inside a `const` block there. The evaluator never
    /// runs a closure (a `LazyLock` init closure is a fn pointer value,
    /// `consteval::ConstValue::FnPtr`); the closure body checks as a body
    /// with reads (check/const_body.rs).
    quals: Option<&'a mut Vec<(Sym, Sym)>>,
    /// Closure bodies entered with `quals` set, minus the `const` blocks
    /// entered inside them: while above 0, `sym` records nothing.
    muted: u32,
    /// With `quals`: only method calls go there; paths record their names
    /// (the roots other than item values, `ConstRoots::scalar_calls`).
    calls_only: bool,
    /// With `quals`: the names that cannot name an associated fn, in place
    /// of `idents`: a one-segment path expression (a local, a const, a free
    /// fn), a field name, and a binding (`ConstReach::reaches_inherent`).
    bare: Vec<Sym>,
    /// The macro hygiene contexts this walk met, in first-seen order
    /// (`ctxt`).
    ctxts: Vec<u32>,
}

impl Walker<'_> {
    /// The hygiene context of a name that resolves among locals and labels
    /// (a path expression, a binding pattern, a label): the call site as 0,
    /// a macro's context by the order this walk first met it. A macro whose
    /// binder flips from `$x` to `x` changes which local its body reads with
    /// the same names and shapes (Sol's const-body review round 2: a false
    /// accept in fn bodies too); raw context ids follow expansion order, so
    /// only this numbering hashes.
    fn ctxt(&mut self, h: &mut FxHasher, c: crate::diag::CtxtId) {
        if c == crate::diag::CtxtId::ROOT {
            return 0u32.hash(h);
        }
        let k = match self.ctxts.iter().position(|&x| x == c.0) {
            Some(k) => k,
            None => {
                self.ctxts.push(c.0);
                self.ctxts.len() - 1
            }
        };
        (k as u32 + 1).hash(h);
    }

    fn sym(&mut self, h: &mut FxHasher, s: Sym) {
        s.hash(h);
        if self.muted > 0 {
            return;
        }
        if let Some(v) = &mut self.idents {
            v.push(s);
        }
    }

    /// `sym` for a name that cannot name an associated fn (`bare`).
    fn bare_sym(&mut self, h: &mut FxHasher, s: Sym) {
        if self.quals.is_some() && self.muted == 0 {
            s.hash(h);
            self.bare.push(s);
        } else {
            self.sym(h, s);
        }
    }

    fn string_at(&mut self, h: &mut FxHasher, i: u32) {
        self.ast.strings.get(i as usize).map_or("", String::as_str).hash(h);
    }

    fn lifetime(&mut self, h: &mut FxHasher, lt: &Lifetime) {
        self.sym(h, lt.name);
    }

    fn vis(&mut self, h: &mut FxHasher, v: &Vis) {
        match v {
            Vis::Private => 0u8.hash(h),
            Vis::Public => 1u8.hash(h),
            Vis::Restricted { path, .. } => {
                2u8.hash(h);
                self.path(h, path);
            }
        }
    }

    fn trees(&mut self, h: &mut FxHasher, trees: &[TokenTree]) {
        (trees.len() as u64).hash(h);
        for t in trees {
            match t {
                TokenTree::Token(tok) => {
                    0u8.hash(h);
                    std::mem::discriminant(&tok.tok).hash(h);
                    match &tok.tok {
                        Tok::Ident(s) | Tok::Reserved(s) | Tok::Lifetime(s) => self.sym(h, *s),
                        Tok::Int(v, suf, _) => {
                            v.hash(h);
                            suf.map(|s| s as u8).hash(h);
                        }
                        Tok::Char(c, _) => c.hash(h),
                        _ => {}
                    }
                    // Strings, including a literal's written text.
                    for i in tok.tok.string_indices() {
                        self.string_at(h, i);
                    }
                }
                TokenTree::Delim(d, inner, _) => {
                    1u8.hash(h);
                    std::mem::discriminant(d).hash(h);
                    self.trees(h, inner);
                }
            }
        }
    }

    fn attrs(&mut self, h: &mut FxHasher, attrs: &[Attr]) {
        (attrs.len() as u64).hash(h);
        for a in attrs {
            match a.style {
                AttrStyle::Outer => 0u8.hash(h),
                AttrStyle::Inner => 1u8.hash(h),
            }
            self.path(h, &a.path);
            match &a.input {
                AttrInput::None => 0u8.hash(h),
                AttrInput::Eq(e) => {
                    1u8.hash(h);
                    self.expr(h, *e);
                }
                AttrInput::List(d, trees) => {
                    2u8.hash(h);
                    std::mem::discriminant(d).hash(h);
                    self.trees(h, trees);
                }
            }
        }
    }

    fn path(&mut self, h: &mut FxHasher, segs: &[PathSeg]) {
        (segs.len() as u64).hash(h);
        for (i, s) in segs.iter().enumerate() {
            // A qualifier with generic arguments (`Id::<T>::f`) may be an
            // alias that denotes any type: the name counts unqualified.
            match &mut self.quals {
                Some(q) if i > 0 && i + 1 == segs.len() && self.muted == 0 && !self.calls_only && matches!(segs[i - 1].args, GenericArgs::None) => {
                    s.name.hash(h);
                    q.push((segs[i - 1].name, s.name));
                }
                _ => self.sym(h, s.name),
            }
            self.generic_args(h, &s.args);
        }
    }

    fn generic_args(&mut self, h: &mut FxHasher, args: &GenericArgs) {
        match args {
            GenericArgs::None => 0u8.hash(h),
            GenericArgs::Angle(args) => {
                1u8.hash(h);
                (args.len() as u64).hash(h);
                for a in args {
                    self.generic_arg(h, a);
                }
            }
            GenericArgs::Parenthesized { inputs, output } => {
                2u8.hash(h);
                for t in inputs {
                    self.ty(h, *t);
                }
                match output {
                    Some(t) => self.ty(h, *t),
                    None => 0u8.hash(h),
                }
            }
        }
    }

    fn generic_arg(&mut self, h: &mut FxHasher, a: &GenericArg) {
        match a {
            GenericArg::Type(t) => {
                0u8.hash(h);
                self.ty(h, *t);
            }
            GenericArg::Lifetime(lt) => {
                1u8.hash(h);
                self.lifetime(h, lt);
            }
            GenericArg::Const(e) => {
                2u8.hash(h);
                self.expr(h, *e);
            }
            GenericArg::Binding { name, kind, .. } => {
                3u8.hash(h);
                self.sym(h, *name);
                match kind {
                    BindingKind::EqTy(t) => {
                        0u8.hash(h);
                        self.ty(h, *t);
                    }
                    BindingKind::EqConst(e) => {
                        1u8.hash(h);
                        self.expr(h, *e);
                    }
                    BindingKind::Constraint(bounds) => {
                        2u8.hash(h);
                        self.bounds(h, bounds);
                    }
                }
            }
        }
    }

    fn bounds(&mut self, h: &mut FxHasher, bounds: &[TraitBound]) {
        (bounds.len() as u64).hash(h);
        for b in bounds {
            match b {
                TraitBound::Trait { binder, trait_ref, maybe } => {
                    0u8.hash(h);
                    for lt in binder {
                        self.lifetime(h, lt);
                    }
                    self.path(h, &trait_ref.path);
                    maybe.hash(h);
                }
                TraitBound::Lifetime(lt) => {
                    1u8.hash(h);
                    self.lifetime(h, lt);
                }
            }
        }
    }

    fn generics(&mut self, h: &mut FxHasher, g: &Generics) {
        (g.params.len() as u64).hash(h);
        for p in &g.params {
            match p {
                GenericParam::Lifetime { lt, bounds, attrs } => {
                    0u8.hash(h);
                    self.lifetime(h, lt);
                    for b in bounds {
                        self.lifetime(h, b);
                    }
                    self.attrs(h, attrs);
                }
                GenericParam::Type { name, bounds, default, attrs, .. } => {
                    1u8.hash(h);
                    self.sym(h, *name);
                    self.bounds(h, bounds);
                    match default {
                        Some(t) => self.ty(h, *t),
                        None => 0u8.hash(h),
                    }
                    self.attrs(h, attrs);
                }
                GenericParam::Const { name, ty, default, attrs, .. } => {
                    2u8.hash(h);
                    self.sym(h, *name);
                    self.ty(h, *ty);
                    match default {
                        Some(e) => self.expr(h, *e),
                        None => 0u8.hash(h),
                    }
                    self.attrs(h, attrs);
                }
            }
        }
    }

    fn where_clause(&mut self, h: &mut FxHasher, w: &WhereClause) {
        (w.preds.len() as u64).hash(h);
        for p in &w.preds {
            match p {
                WherePred::Type { binder, ty, bounds } => {
                    0u8.hash(h);
                    for lt in binder {
                        self.lifetime(h, lt);
                    }
                    self.ty(h, *ty);
                    self.bounds(h, bounds);
                }
                WherePred::Lifetime { lt, bounds } => {
                    1u8.hash(h);
                    self.lifetime(h, lt);
                    for b in bounds {
                        self.lifetime(h, b);
                    }
                }
            }
        }
    }

    fn qual(&mut self, h: &mut FxHasher, q: &FnQual) {
        (q.is_const, q.is_async, q.is_unsafe).hash(h);
        match q.abi {
            Some(i) => self.string_at(h, i),
            None => 0u8.hash(h),
        }
    }

    fn ty(&mut self, h: &mut FxHasher, t: TyId) {
        match self.ast.ty(t) {
            TyExpr::Path(segs) => {
                0u8.hash(h);
                self.path(h, segs);
            }
            TyExpr::Ref { lifetime, mutable, inner } => {
                1u8.hash(h);
                match lifetime {
                    Some(lt) => self.lifetime(h, lt),
                    None => 0u8.hash(h),
                }
                mutable.hash(h);
                self.ty(h, *inner);
            }
            TyExpr::Unit => 2u8.hash(h),
            TyExpr::Tuple(ts) => {
                3u8.hash(h);
                (ts.len() as u64).hash(h);
                for t in ts {
                    self.ty(h, *t);
                }
            }
            TyExpr::Array { inner, len } => {
                4u8.hash(h);
                self.ty(h, *inner);
                self.expr(h, *len);
            }
            TyExpr::Slice(inner) => {
                5u8.hash(h);
                self.ty(h, *inner);
            }
            TyExpr::Ptr { mutable, inner } => {
                (6u8, mutable).hash(h);
                self.ty(h, *inner);
            }
            TyExpr::FnPtr { binder, qual, inputs, variadic, ret } => {
                7u8.hash(h);
                for lt in binder {
                    self.lifetime(h, lt);
                }
                self.qual(h, qual);
                (inputs.len() as u64).hash(h);
                for p in inputs {
                    match p.name {
                        Some(n) => self.sym(h, n),
                        None => 0u8.hash(h),
                    }
                    self.attrs(h, &p.attrs);
                    self.ty(h, p.ty);
                }
                variadic.hash(h);
                match ret {
                    Some(t) => self.ty(h, *t),
                    None => 0u8.hash(h),
                }
            }
            TyExpr::Dyn { binder, bounds, lifetime } => {
                8u8.hash(h);
                for lt in binder {
                    self.lifetime(h, lt);
                }
                self.bounds(h, bounds);
                match lifetime {
                    Some(lt) => self.lifetime(h, lt),
                    None => 0u8.hash(h),
                }
            }
            TyExpr::ImplTrait { bounds, captures } => {
                9u8.hash(h);
                self.bounds(h, bounds);
                captures.as_ref().map(|c| c.args.len()).hash(h);
                for a in captures.iter().flat_map(|c| &c.args) {
                    match a {
                        UseArg::Lifetime(lt) => {
                            0u8.hash(h);
                            self.lifetime(h, lt);
                        }
                        UseArg::Param(n, _) => {
                            1u8.hash(h);
                            self.sym(h, *n);
                        }
                    }
                }
            }
            TyExpr::QPath { self_ty, trait_path, segs } => {
                10u8.hash(h);
                self.ty(h, *self_ty);
                self.path(h, trait_path);
                self.path(h, segs);
            }
            TyExpr::Never => 11u8.hash(h),
            TyExpr::Paren(inner) => {
                12u8.hash(h);
                self.ty(h, *inner);
            }
            TyExpr::Infer => 13u8.hash(h),
            TyExpr::MacroCall(c) => {
                14u8.hash(h);
                self.macro_call(h, c);
            }
        }
    }

    fn macro_call(&mut self, h: &mut FxHasher, c: &MacroCall) {
        self.path(h, &c.path);
        std::mem::discriminant(&c.delim).hash(h);
        self.trees(h, &c.tokens);
    }

    fn block(&mut self, h: &mut FxHasher, b: BlockId) {
        let block = self.ast.block(b);
        self.attrs(h, &block.inner_attrs);
        (block.stmts.len() as u64).hash(h);
        for s in &block.stmts {
            self.stmt(h, s);
        }
        self.attrs(h, &block.tail_attrs);
        match block.tail {
            Some(e) => self.expr(h, e),
            None => 0u8.hash(h),
        }
    }

    fn stmt(&mut self, h: &mut FxHasher, s: &Stmt) {
        match s {
            Stmt::Let { pat, ty, init, els, attrs, .. } => {
                0u8.hash(h);
                self.pat(h, *pat);
                match ty {
                    Some(t) => self.ty(h, *t),
                    None => 0u8.hash(h),
                }
                match init {
                    Some(e) => self.expr(h, *e),
                    None => 0u8.hash(h),
                }
                match els {
                    Some(b) => self.block(h, *b),
                    None => 0u8.hash(h),
                }
                self.attrs(h, attrs);
            }
            Stmt::Expr { expr, semi, attrs } => {
                (1u8, semi).hash(h);
                self.expr(h, *expr);
                self.attrs(h, attrs);
            }
            Stmt::Item(i) => {
                2u8.hash(h);
                if let Some(inner) = self.inner.as_deref_mut() {
                    inner.push(*i);
                }
                // A body-local item is part of this body's meaning:
                // hash it whole, fn bodies included.
                let item = &self.ast.stmt_items[*i as usize];
                self.item(h, item, true);
            }
            Stmt::Macro { call, semi } => {
                (3u8, semi).hash(h);
                self.macro_call(h, call);
            }
        }
    }

    fn label(&mut self, h: &mut FxHasher, l: &Option<Label>) {
        match l {
            Some(l) => {
                self.lifetime(h, l);
                self.ctxt(h, l.span.ctxt);
            }
            None => 0u8.hash(h),
        }
    }

    fn expr(&mut self, h: &mut FxHasher, e: ExprId) {
        match self.ast.expr(e) {
            Expr::Int(v, suf) => {
                0u8.hash(h);
                v.hash(h);
                suf.map(|s| s as u8).hash(h);
            }
            Expr::Bool(b) => (1u8, b).hash(h),
            Expr::Str(i) => {
                2u8.hash(h);
                self.string_at(h, *i);
            }
            Expr::Char(c) => (3u8, c).hash(h),
            Expr::Unit => 4u8.hash(h),
            Expr::Path(segs) => {
                5u8.hash(h);
                self.ctxt(h, self.ast.expr_span(e).ctxt);
                match segs.as_slice() {
                    [s] if matches!(s.args, GenericArgs::None) => {
                        1u64.hash(h);
                        self.bare_sym(h, s.name);
                        self.generic_args(h, &s.args);
                    }
                    _ => self.path(h, segs),
                }
            }
            Expr::SelfValue => 6u8.hash(h),
            Expr::Block(b) => {
                7u8.hash(h);
                self.block(h, *b);
            }
            Expr::If { cond, then, els } => {
                8u8.hash(h);
                self.expr(h, *cond);
                self.block(h, *then);
                match els {
                    Some(e) => self.expr(h, *e),
                    None => 0u8.hash(h),
                }
            }
            Expr::While { label, cond, body } => {
                9u8.hash(h);
                self.label(h, label);
                self.expr(h, *cond);
                self.block(h, *body);
            }
            Expr::Loop { label, body } => {
                10u8.hash(h);
                self.label(h, label);
                self.block(h, *body);
            }
            Expr::For { label, pat, iter, body } => {
                11u8.hash(h);
                self.label(h, label);
                self.pat(h, *pat);
                self.expr(h, *iter);
                self.block(h, *body);
            }
            Expr::Match { scrut, arms } => {
                12u8.hash(h);
                self.expr(h, *scrut);
                (arms.len() as u64).hash(h);
                for a in arms {
                    self.attrs(h, &a.attrs);
                    self.pat(h, a.pat);
                    match a.guard {
                        Some(g) => self.expr(h, g),
                        None => 0u8.hash(h),
                    }
                    self.expr(h, a.body);
                }
            }
            Expr::Call { callee, args } => {
                13u8.hash(h);
                self.expr(h, *callee);
                (args.len() as u64).hash(h);
                for a in args {
                    self.expr(h, *a);
                }
            }
            Expr::MethodCall { recv, name, generic_args, args, .. } => {
                14u8.hash(h);
                self.expr(h, *recv);
                // A root's method call gives (name, name) in `quals`: its
                // receiver type is unknown by name (`reaches_inherent`).
                match &mut self.quals {
                    Some(q) if self.muted == 0 => {
                        name.hash(h);
                        q.push((*name, *name));
                    }
                    _ => self.sym(h, *name),
                }
                (generic_args.len() as u64).hash(h);
                for a in generic_args {
                    self.generic_arg(h, a);
                }
                (args.len() as u64).hash(h);
                for a in args {
                    self.expr(h, *a);
                }
            }
            Expr::Field { base, name } => {
                15u8.hash(h);
                self.expr(h, *base);
                self.bare_sym(h, *name);
            }
            Expr::Index { base, index } => {
                16u8.hash(h);
                self.expr(h, *base);
                self.expr(h, *index);
            }
            Expr::Binary { op, lhs, rhs } => {
                17u8.hash(h);
                std::mem::discriminant(op).hash(h);
                self.expr(h, *lhs);
                self.expr(h, *rhs);
            }
            Expr::Unary { op, expr } => {
                18u8.hash(h);
                std::mem::discriminant(op).hash(h);
                self.expr(h, *expr);
            }
            Expr::Cast { expr, ty } => {
                19u8.hash(h);
                self.expr(h, *expr);
                self.ty(h, *ty);
            }
            Expr::Ref { mutable, raw, expr } => {
                (20u8, mutable, raw).hash(h);
                self.expr(h, *expr);
            }
            Expr::Deref(e) => {
                21u8.hash(h);
                self.expr(h, *e);
            }
            Expr::Assign { lhs, rhs } => {
                22u8.hash(h);
                self.expr(h, *lhs);
                self.expr(h, *rhs);
            }
            Expr::AssignOp { op, lhs, rhs } => {
                23u8.hash(h);
                std::mem::discriminant(op).hash(h);
                self.expr(h, *lhs);
                self.expr(h, *rhs);
            }
            Expr::StructLit { path, fields, update } => {
                24u8.hash(h);
                self.path(h, path);
                (fields.len() as u64).hash(h);
                for (name, _, e) in fields {
                    self.bare_sym(h, *name);
                    self.expr(h, *e);
                }
                match update {
                    Some(e) => self.expr(h, *e),
                    None => 0u8.hash(h),
                }
            }
            Expr::Range { lo, hi, inclusive } => {
                (25u8, inclusive).hash(h);
                match lo {
                    Some(e) => self.expr(h, *e),
                    None => 0u8.hash(h),
                }
                match hi {
                    Some(e) => self.expr(h, *e),
                    None => 0u8.hash(h),
                }
            }
            Expr::Break { label, value } => {
                26u8.hash(h);
                self.label(h, label);
                match value {
                    Some(e) => self.expr(h, *e),
                    None => 0u8.hash(h),
                }
            }
            Expr::Continue(label) => {
                27u8.hash(h);
                self.label(h, label);
            }
            Expr::Return(v) => {
                28u8.hash(h);
                match v {
                    Some(e) => self.expr(h, *e),
                    None => 0u8.hash(h),
                }
            }
            Expr::Macro { kind, args, fmt, repeat } => {
                (29u8, repeat).hash(h);
                std::mem::discriminant(kind).hash(h);
                (args.len() as u64).hash(h);
                for a in args {
                    self.expr(h, *a);
                }
                (fmt.len() as u64).hash(h);
                for f in fmt {
                    match f {
                        FmtPiece::Lit(s) => {
                            0u8.hash(h);
                            s.hash(h);
                        }
                        FmtPiece::Arg { index, debug, spec } => {
                            (1u8, *index as u64, debug).hash(h);
                            spec.hash(h);
                        }
                    }
                }
            }
            Expr::Float(i) => {
                30u8.hash(h);
                self.string_at(h, *i);
            }
            Expr::Tuple(es) => {
                31u8.hash(h);
                (es.len() as u64).hash(h);
                for e in es {
                    self.expr(h, *e);
                }
            }
            Expr::Array(es) => {
                32u8.hash(h);
                (es.len() as u64).hash(h);
                for e in es {
                    self.expr(h, *e);
                }
            }
            Expr::ArrayRepeat { value, len } => {
                33u8.hash(h);
                self.expr(h, *value);
                self.expr(h, *len);
            }
            Expr::Closure { is_move, is_async, params, ret, body } => {
                (34u8, is_move, is_async).hash(h);
                (params.len() as u64).hash(h);
                for p in params {
                    self.pat(h, p.pat);
                    match p.ty {
                        Some(t) => self.ty(h, t),
                        None => 0u8.hash(h),
                    }
                    self.attrs(h, &p.attrs);
                }
                match ret {
                    Some(t) => self.ty(h, *t),
                    None => 0u8.hash(h),
                }
                let roots = u32::from(self.quals.is_some());
                self.muted += roots;
                self.expr(h, *body);
                self.muted -= roots;
            }
            Expr::Try(e) => {
                35u8.hash(h);
                self.expr(h, *e);
            }
            Expr::Await(e) => {
                36u8.hash(h);
                self.expr(h, *e);
            }
            Expr::AsyncBlock { is_move, body } => {
                (37u8, is_move).hash(h);
                self.block(h, *body);
            }
            Expr::UnsafeBlock(b) => {
                38u8.hash(h);
                self.block(h, *b);
            }
            Expr::LabeledBlock { label, body } => {
                39u8.hash(h);
                self.lifetime(h, label);
                self.block(h, *body);
            }
            Expr::Let { pat, scrut } => {
                40u8.hash(h);
                self.pat(h, *pat);
                self.expr(h, *scrut);
            }
            Expr::TupleField { base, index } => {
                (41u8, index).hash(h);
                self.expr(h, *base);
            }
            Expr::QPath { self_ty, trait_path, segs } => {
                42u8.hash(h);
                self.ty(h, *self_ty);
                self.path(h, trait_path);
                self.path(h, segs);
            }
            Expr::ConstBlock(b) => {
                43u8.hash(h);
                let muted = std::mem::take(&mut self.muted);
                self.block(h, *b);
                self.muted = muted;
            }
            Expr::Underscore => 44u8.hash(h),
            Expr::MacroCall(c) => {
                45u8.hash(h);
                self.macro_call(h, c);
            }
            Expr::OffsetOf { ty, fields } => {
                46u8.hash(h);
                self.ty(h, *ty);
                (fields.len() as u64).hash(h);
                for f in fields {
                    match f {
                        crate::ast::OffsetField::Name(s) => {
                            0u8.hash(h);
                            self.sym(h, *s);
                        }
                        crate::ast::OffsetField::Index(i) => {
                            1u8.hash(h);
                            i.hash(h);
                        }
                    }
                }
            }
            Expr::ByteStr(i) => {
                47u8.hash(h);
                // The bytes themselves: an include_bytes! file edit
                // changes them under the same index.
                match self.ast.blobs.get(*i as usize) {
                    Some(b) => b.as_slice().hash(h),
                    None => 0u64.hash(h),
                }
            }
        }
    }

    fn pat(&mut self, h: &mut FxHasher, p: PatId) {
        match self.ast.pat(p) {
            Pat::Wild => 0u8.hash(h),
            Pat::Binding { name, mutable, by_ref } => {
                (1u8, mutable, by_ref).hash(h);
                self.ctxt(h, self.ast.pat_span(p).ctxt);
                self.bare_sym(h, *name);
            }
            Pat::Lit(e) => {
                2u8.hash(h);
                self.expr(h, *e);
            }
            Pat::Path(segs) => {
                3u8.hash(h);
                self.path(h, segs);
            }
            Pat::TupleStruct { path, elems } => {
                4u8.hash(h);
                self.path(h, path);
                (elems.len() as u64).hash(h);
                for e in elems {
                    self.pat(h, *e);
                }
            }
            Pat::Struct { path, fields, rest } => {
                (5u8, rest).hash(h);
                self.path(h, path);
                (fields.len() as u64).hash(h);
                for (name, sub) in fields {
                    self.sym(h, *name);
                    self.pat(h, *sub);
                }
            }
            Pat::Or(elems) => {
                6u8.hash(h);
                (elems.len() as u64).hash(h);
                for e in elems {
                    self.pat(h, *e);
                }
            }
            Pat::Range { lo, hi, inclusive } => {
                (7u8, inclusive).hash(h);
                match lo {
                    Some(e) => self.expr(h, *e),
                    None => 0u8.hash(h),
                }
                match hi {
                    Some(e) => self.expr(h, *e),
                    None => 0u8.hash(h),
                }
            }
            Pat::Tuple(elems) => {
                8u8.hash(h);
                (elems.len() as u64).hash(h);
                for e in elems {
                    self.pat(h, *e);
                }
            }
            Pat::Slice { prefix, middle, suffix } => {
                9u8.hash(h);
                (prefix.len() as u64).hash(h);
                for e in prefix {
                    self.pat(h, *e);
                }
                match middle {
                    Some(m) => self.pat(h, *m),
                    None => 0u8.hash(h),
                }
                (suffix.len() as u64).hash(h);
                for e in suffix {
                    self.pat(h, *e);
                }
            }
            Pat::Ref { mutable, inner } => {
                (10u8, mutable).hash(h);
                self.pat(h, *inner);
            }
            Pat::At { name, mutable, by_ref, inner } => {
                (11u8, mutable, by_ref).hash(h);
                self.sym(h, *name);
                self.pat(h, *inner);
            }
            Pat::Rest => 12u8.hash(h),
            Pat::Paren(inner) => {
                13u8.hash(h);
                self.pat(h, *inner);
            }
            Pat::QPath { self_ty, trait_path, segs } => {
                14u8.hash(h);
                self.ty(h, *self_ty);
                self.path(h, trait_path);
                self.path(h, segs);
            }
            Pat::MacroCall(c) => {
                15u8.hash(h);
                self.macro_call(h, c);
            }
        }
    }
}

impl Walker<'_> {
    /// A fn's signature side: everything but the body block, unless
    /// `with_body` (trait default bodies and body-local items are
    /// environment, not separately numbered fns).
    fn fn_head(&mut self, h: &mut FxHasher, f: &FnDef, with_body: bool) {
        self.sym(h, f.name);
        f.is_test.hash(h);
        self.vis(h, &f.vis);
        self.attrs(h, &f.attrs);
        self.qual(h, &f.qual);
        self.generics(h, &f.generics);
        match &f.self_param {
            Some(sp) => {
                1u8.hash(h);
                match sp.kind {
                    SelfKind::Value => 0u8.hash(h),
                    SelfKind::Ref => 1u8.hash(h),
                    SelfKind::RefMut => 2u8.hash(h),
                    SelfKind::Typed => 3u8.hash(h),
                }
                sp.mutable.hash(h);
                match &sp.lifetime {
                    Some(lt) => self.lifetime(h, lt),
                    None => 0u8.hash(h),
                }
                match sp.ty {
                    Some(t) => self.ty(h, t),
                    None => 0u8.hash(h),
                }
                self.attrs(h, &sp.attrs);
            }
            None => 0u8.hash(h),
        }
        (f.params.len() as u64).hash(h);
        for p in &f.params {
            self.sym(h, p.name);
            p.mutable.hash(h);
            match p.pat {
                Some(pat) => self.pat(h, pat),
                None => 0u8.hash(h),
            }
            self.ty(h, p.ty);
            self.attrs(h, &p.attrs);
        }
        f.variadic.hash(h);
        match f.ret {
            Some(t) => self.ty(h, t),
            None => 0u8.hash(h),
        }
        self.where_clause(h, &f.where_clause);
        if with_body {
            match f.body {
                Some(b) => self.block(h, b),
                None => 0u8.hash(h),
            }
        }
    }

    fn fields(&mut self, h: &mut FxHasher, fields: &VariantFields) {
        let (tag, fs): (u8, &[FieldDef]) = match fields {
            VariantFields::Unit => (0, &[]),
            VariantFields::Tuple(fs) => (1, fs),
            VariantFields::Struct(fs) => (2, fs),
        };
        tag.hash(h);
        (fs.len() as u64).hash(h);
        for fd in fs {
            self.sym(h, fd.name);
            self.vis(h, &fd.vis);
            self.attrs(h, &fd.attrs);
            self.ty(h, fd.ty);
        }
    }

    fn assoc_consts(&mut self, h: &mut FxHasher, consts: &[AssocConstDef]) {
        (consts.len() as u64).hash(h);
        for c in consts {
            self.sym(h, c.name);
            self.vis(h, &c.vis);
            self.attrs(h, &c.attrs);
            self.ty(h, c.ty);
            match c.default {
                Some(e) => self.expr(h, e),
                None => 0u8.hash(h),
            }
        }
    }

    fn assoc_types(&mut self, h: &mut FxHasher, types: &[AssocTypeDef]) {
        (types.len() as u64).hash(h);
        for t in types {
            self.sym(h, t.name);
            self.vis(h, &t.vis);
            self.attrs(h, &t.attrs);
            self.generics(h, &t.generics);
            self.bounds(h, &t.bounds);
            self.where_clause(h, &t.where_clause);
            match t.default {
                Some(ty) => self.ty(h, ty),
                None => 0u8.hash(h),
            }
        }
    }

    fn use_tree(&mut self, h: &mut FxHasher, t: &UseTree) {
        t.leading_colon.hash(h);
        self.path(h, &t.prefix);
        match &t.kind {
            UseKind::Simple { rename } => {
                0u8.hash(h);
                match rename {
                    Some(r) => self.sym(h, *r),
                    None => 0u8.hash(h),
                }
            }
            UseKind::Glob => 1u8.hash(h),
            UseKind::Group(children) => {
                2u8.hash(h);
                (children.len() as u64).hash(h);
                for c in children {
                    self.use_tree(h, c);
                }
            }
        }
    }

    /// One item's environment hash contribution. `with_bodies` includes
    /// fn body blocks (body-local items only; top-level fn bodies are
    /// hashed per fn by `body_scan`).
    fn item(&mut self, h: &mut FxHasher, item: &Item, with_bodies: bool) {
        self.vis(h, &item.vis);
        self.attrs(h, &item.attrs);
        match &item.kind {
            ItemKind::Fn(f) => {
                0u8.hash(h);
                self.fn_head(h, f, with_bodies);
            }
            ItemKind::Struct(s) => {
                1u8.hash(h);
                self.sym(h, s.name);
                self.generics(h, &s.generics);
                self.fields(h, &s.fields);
                self.where_clause(h, &s.where_clause);
                (s.derives.len() as u64).hash(h);
                for (d, _) in &s.derives {
                    self.sym(h, *d);
                }
            }
            ItemKind::Enum(e) => {
                2u8.hash(h);
                self.sym(h, e.name);
                self.generics(h, &e.generics);
                (e.variants.len() as u64).hash(h);
                for v in &e.variants {
                    self.sym(h, v.name);
                    self.attrs(h, &v.attrs);
                    self.fields(h, &v.fields);
                    match v.discriminant {
                        Some(x) => self.expr(h, x),
                        None => 0u8.hash(h),
                    }
                }
                self.where_clause(h, &e.where_clause);
                (e.derives.len() as u64).hash(h);
                for (d, _) in &e.derives {
                    self.sym(h, *d);
                }
            }
            ItemKind::Impl(imp) => {
                3u8.hash(h);
                self.attrs(h, &imp.inner_attrs);
                self.generics(h, &imp.generics);
                (imp.is_unsafe, imp.is_negative, imp.in_body).hash(h);
                match &imp.trait_ref {
                    Some(tr) => self.path(h, &tr.path),
                    None => 0u8.hash(h),
                }
                self.ty(h, imp.self_ty);
                self.where_clause(h, &imp.where_clause);
                (imp.fns.len() as u64).hash(h);
                for f in &imp.fns {
                    self.fn_head(h, f, with_bodies);
                }
                self.assoc_consts(h, &imp.consts);
                self.assoc_types(h, &imp.types);
            }
            ItemKind::Use(u) => {
                4u8.hash(h);
                self.use_tree(h, &u.tree);
            }
            ItemKind::Mod(m) => {
                5u8.hash(h);
                self.attrs(h, &m.inner_attrs);
                self.sym(h, m.name);
                match &m.body {
                    Some(items) => {
                        (items.len() as u64).hash(h);
                        for it in items {
                            self.item(h, it, with_bodies);
                        }
                    }
                    None => 0u8.hash(h),
                }
            }
            ItemKind::Trait(t) => {
                6u8.hash(h);
                self.attrs(h, &t.inner_attrs);
                self.sym(h, t.name);
                (t.is_unsafe, t.is_auto).hash(h);
                self.generics(h, &t.generics);
                self.bounds(h, &t.bounds);
                self.where_clause(h, &t.where_clause);
                (t.fns.len() as u64).hash(h);
                for f in &t.fns {
                    // Trait default bodies are environment: they are
                    // not separately numbered fns.
                    self.fn_head(h, f, true);
                }
                self.assoc_consts(h, &t.consts);
                self.assoc_types(h, &t.types);
            }
            ItemKind::Type(a) => {
                7u8.hash(h);
                self.sym(h, a.name);
                self.generics(h, &a.generics);
                self.where_clause(h, &a.where_clause);
                self.ty(h, a.ty);
            }
            ItemKind::Const(c) => {
                8u8.hash(h);
                match c.name {
                    Some(n) => self.sym(h, n),
                    None => 0u8.hash(h),
                }
                self.ty(h, c.ty);
                match c.value {
                    Some(e) => self.expr(h, e),
                    None => 0u8.hash(h),
                }
            }
            ItemKind::Static(s) => {
                (9u8, s.mutable).hash(h);
                self.sym(h, s.name);
                self.ty(h, s.ty);
                match s.value {
                    Some(e) => self.expr(h, e),
                    None => 0u8.hash(h),
                }
            }
            ItemKind::MacroRules(m) => {
                10u8.hash(h);
                self.sym(h, m.name);
                std::mem::discriminant(&m.delim).hash(h);
                self.trees(h, &m.tokens);
            }
            ItemKind::ExternCrate(e) => {
                11u8.hash(h);
                self.sym(h, e.name);
                match e.rename {
                    Some(r) => self.sym(h, r),
                    None => 0u8.hash(h),
                }
            }
            ItemKind::ExternBlock(b) => {
                (12u8, b.is_unsafe).hash(h);
                match b.abi {
                    Some(i) => self.string_at(h, i),
                    None => 0u8.hash(h),
                }
                (b.items.len() as u64).hash(h);
                for it in &b.items {
                    match it {
                        ForeignItem::Fn(f) => {
                            0u8.hash(h);
                            self.fn_head(h, f, with_bodies);
                        }
                        ForeignItem::Static(s) => {
                            (1u8, s.mutable).hash(h);
                            self.sym(h, s.name);
                            self.ty(h, s.ty);
                            match s.value {
                                Some(e) => self.expr(h, e),
                                None => 0u8.hash(h),
                            }
                        }
                        ForeignItem::Type { name, vis, attrs, generics, where_clause, .. } => {
                            2u8.hash(h);
                            self.sym(h, *name);
                            self.vis(h, vis);
                            self.attrs(h, attrs);
                            self.generics(h, generics);
                            self.where_clause(h, where_clause);
                        }
                        ForeignItem::MacroCall(c) => {
                            3u8.hash(h);
                            self.macro_call(h, c);
                        }
                    }
                }
            }
            ItemKind::Union(u) => {
                13u8.hash(h);
                self.sym(h, u.name);
                self.generics(h, &u.generics);
                (u.fields.len() as u64).hash(h);
                for fd in &u.fields {
                    self.sym(h, fd.name);
                    self.vis(h, &fd.vis);
                    self.attrs(h, &fd.attrs);
                    self.ty(h, fd.ty);
                }
                self.where_clause(h, &u.where_clause);
                (u.derives.len() as u64).hash(h);
                for (d, _) in &u.derives {
                    self.sym(h, *d);
                }
            }
            ItemKind::MacroCall(c) => {
                14u8.hash(h);
                self.macro_call(h, c);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diag::FileId;
    use crate::intern::{Interner, Known};
    use crate::{lexer, parser};

    fn scan_src(src: &str) -> AstScans {
        let interner = Interner::default();
        let known = Known::new(&interner);
        let lexed = lexer::lex(src, FileId(0), 0, &interner).unwrap();
        let ast = parser::parse(&lexed, &interner, &known).unwrap();
        scan_ast(&ast)
    }

    /// Per-chunk serial scans concatenate to the merged Ast scan:
    /// the worklist caches one scan per file chunk and joins them.
    #[test]
    fn chunk_scans_join_to_merged_scan() {
        let interner = Interner::default();
        let known = Known::new(&interner);
        let parse = |src: &str, file: u32| {
            let lexed = lexer::lex(src, FileId(file), 0, &interner).unwrap();
            parser::parse(&lexed, &interner, &known).unwrap()
        };
        let a = parse("struct A { x: i64 }\nfn fa(a: i64) -> i64 { a + 1 }\n", 0);
        let b = parse("fn fb() -> &'static str { \"hi\" }\nfn main() { println!(\"{}\", fa(2)); }\n", 1);
        let (sa, sb) = (scan_ast_seq(&a), scan_ast_seq(&b));
        let merged = scan_ast(&Ast::concat(vec![a, b]));
        let joined_items: Vec<u64> = sa.items.iter().chain(&sb.items).map(|i| i.hash).collect();
        assert_eq!(joined_items, merged.items.iter().map(|i| i.hash).collect::<Vec<_>>());
        let joined_bodies: Vec<u64> = sa.bodies.iter().chain(&sb.bodies).map(|b| b.0).collect();
        assert_eq!(joined_bodies, merged.bodies.iter().map(|b| b.0).collect::<Vec<_>>());
        let joined_idents: Vec<&Vec<Sym>> = sa.bodies.iter().chain(&sb.bodies).map(|b| &b.1).collect();
        assert_eq!(joined_idents, merged.bodies.iter().map(|b| &b.1).collect::<Vec<_>>());
    }

    const BASE: &str = "\
struct Foo { x: i64 }
impl Foo { fn get(&self) -> i64 { self.x } }
fn helper(a: i64) -> i64 { a + 1 }
fn main() { let f = Foo { x: helper(1) }; println!(\"{}\", f.get()); }
";

    /// A body-only edit changes exactly that body's hash; every item
    /// hash and every other body hash stays.
    #[test]
    fn body_edit_changes_only_that_body() {
        let a = scan_src(BASE);
        let b = scan_src(&BASE.replace("a + 1", "a + 2"));
        assert_eq!(a.items.iter().map(|i| i.hash).collect::<Vec<_>>(), b.items.iter().map(|i| i.hash).collect::<Vec<_>>());
        let changed: Vec<usize> = (0..a.bodies.len()).filter(|&i| a.bodies[i].0 != b.bodies[i].0).collect();
        assert_eq!(changed, vec![1], "only helper (fn order: get, helper, main)");
    }

    /// A signature edit changes the item hash, not the body hash.
    #[test]
    fn signature_edit_changes_item_not_body() {
        let a = scan_src(BASE);
        let b = scan_src(&BASE.replace("fn helper(a: i64)", "fn helper(a: i64, extra: i64)"));
        assert_ne!(a.items[2].hash, b.items[2].hash);
        assert_eq!(a.bodies[1].0, b.bodies[1].0);
    }

    /// Moves and blank lines hash the same: positions are not content.
    #[test]
    fn line_shifts_do_not_rehash() {
        let a = scan_src(BASE);
        let b = scan_src(&format!("\n\n{BASE}"));
        assert_eq!(a.items.iter().map(|i| i.hash).collect::<Vec<_>>(), b.items.iter().map(|i| i.hash).collect::<Vec<_>>());
        assert_eq!(a.bodies.iter().map(|b| b.0).collect::<Vec<_>>(), b.bodies.iter().map(|b| b.0).collect::<Vec<_>>());
    }

    /// A type edit (field added) changes the type item's hash only.
    #[test]
    fn type_edit_changes_type_item() {
        let a = scan_src(BASE);
        let b = scan_src(&BASE.replace("struct Foo { x: i64 }", "struct Foo { x: i64, y: i64 }"));
        assert_ne!(a.items[0].hash, b.items[0].hash);
        assert_eq!(a.items[2].hash, b.items[2].hash);
        assert_eq!(a.bodies.iter().map(|b| b.0).collect::<Vec<_>>(), b.bodies.iter().map(|b| b.0).collect::<Vec<_>>());
    }

    /// A nested fn's body hash takes in its top-level fn: an item added
    /// next to it (one that can shadow a name it calls) rehashes it, and
    /// an edit in another top-level fn does not.
    #[test]
    fn nested_fn_hash_covers_its_enclosing_fn() {
        let src = "fn h() -> i32 { 1 }\nfn f() -> i32 {\n    fn g() -> i32 { h() }\n    g()\n}\nfn main() { println!(\"{}\", f()); }\n";
        let a = scan_src(src);
        let b = scan_src(&src.replace("    g()\n", "    fn h() -> i32 { 2 }\n    g()\n"));
        let c = scan_src(&src.replace("{ 1 }", "{ 3 }"));
        // Fn order: h, f, main, then the nested g (and the new h in b).
        assert_ne!(a.bodies[3].0, b.bodies[3].0, "a sibling item rechecks g");
        assert_eq!(a.bodies[3].0, c.bodies[3].0, "another fn's body edit keeps g");
    }
}
