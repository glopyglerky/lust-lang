// rustc's deny-by-default lint `ambiguous_associated_items`: inside a trait
// impl for an enum, the type path `Self::V` names the associated type `V`,
// and the enum also has a variant `V`. rustc resolves the type but denies
// the path (a future error). The value path `Self::V` is the variant and is
// fine, and so is `<Self as Tr>::V`.
use crate::ast::{self, ItemKind, TyExpr};
use crate::check::collect::Program;
use crate::diag::{Diag, Span};
use crate::intern::{FxMap, Sym};
use crate::ty::{TyIntern, TyKind};

/// The lint and the group that holds it: an `allow`, `warn`, or `expect`
/// of either lifts it.
const NAMES: &[&str] = &["ambiguous_associated_items", "future_incompatible"];

/// One trait impl whose Self is a local enum: the names that are both a
/// variant and an associated type of the impl's trait, and the impl item.
struct Cand<'a> {
    names: Vec<Sym>,
    item: &'a ast::Item,
    imp: &'a ast::ImplDef,
}

/// Reports each `Self::V` type path that rustc's lint denies (L0622).
pub fn check(p: &Program, errs: &mut Vec<Diag>) {
    // Local trait impls on enums, keyed by where the impl starts.
    let mut enum_impls: FxMap<(u32, u32), Vec<Sym>> = FxMap::default();
    for (_, rec) in p.gx.index.all_impls() {
        if !rec.local {
            continue;
        }
        let TyKind::Adt(d, _) = p.tys.kind(rec.self_ty(&p.tys)) else { continue };
        let Some(adt) = p.defs.local_adt(d) else { continue };
        let def = p.adt(adt);
        if !def.is_enum {
            continue;
        }
        let Some(trec) = p.gx.index.trait_record(rec.trait_def) else { continue };
        let names: Vec<Sym> =
            def.variants.iter().map(|v| v.name).filter(|n| trec.assoc_items.iter().any(|&(_, a, is_type)| is_type && a == *n)).collect();
        if !names.is_empty() {
            enum_impls.insert((rec.span.file.0, rec.span.lo), names);
        }
    }
    if enum_impls.is_empty() {
        return;
    }
    let mut cands: Vec<Cand> = Vec::new();
    for item in p.ast.items.iter().chain(&p.ast.stmt_items) {
        let ItemKind::Impl(imp) = &item.kind else { continue };
        if imp.trait_ref.is_none() {
            continue;
        }
        let key = |s: Span| (s.file.0, s.lo);
        if let Some(names) = enum_impls.get(&key(imp.span)).or_else(|| enum_impls.get(&key(item.span))) {
            cands.push(Cand { names: names.clone(), item, imp });
        }
    }
    let self_type = p.known.self_type;
    for (i, ty) in p.ast.tys.iter().enumerate() {
        let TyExpr::Path(segs) = ty else { continue };
        if segs.len() != 2 || segs[0].name != self_type {
            continue;
        }
        let span = p.ast.ty_spans[i];
        let Some(c) = cands.iter().find(|c| c.names.contains(&segs[1].name) && within(span, c.item.span)) else { continue };
        if lifted(p, c, span) {
            continue;
        }
        errs.push(Diag::new(
            "L0622",
            span,
            format!(
                "ambiguous associated item: `Self::{}` is both a variant of the enum and an associated type",
                p.interner.get(segs[1].name)
            ),
            format!("Write `<Self as Trait>::{}` for the associated type.", p.interner.get(segs[1].name)),
        ));
        return;
    }
}

fn within(inner: Span, outer: Span) -> bool {
    inner.file == outer.file && outer.lo <= inner.lo && inner.hi <= outer.hi
}

/// True when a lint level above the path lifts the lint. A fn's attributes
/// already hold the `allow`, `warn`, and `expect` attributes of every
/// enclosing scope (the parser's `scope_allows`), so a path in a method
/// reads them; a path elsewhere in the impl reads the impl's own and the
/// crate's (an enclosing module's allow is missed there: a false reject).
fn lifted(p: &Program, c: &Cand, span: Span) -> bool {
    let leveled = |attrs: &[ast::Attr]| crate::check::deny_lints::leveled(p, attrs, NAMES);
    for f in &c.imp.fns {
        let end = f.body.map_or(f.name_span.hi, |b| p.ast.block(b).span.hi);
        if f.name_span.file == span.file && f.name_span.lo <= span.lo && span.hi <= end {
            return leveled(&f.attrs);
        }
    }
    leveled(&c.item.attrs) || leveled(&c.imp.inner_attrs) || leveled(&p.ast.inner_attrs)
}
