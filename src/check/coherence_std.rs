//! Std impls that only coherence sees (`ImplIndex::shadows`): core and
//! alloc impls on the fundamental wrappers (`&T`, `&mut T`, `Box<T>`)
//! and on `Rc<T>` that a user impl can legally name (the orphan rule
//! lets `impl Hash for &Local` and `impl From<Box<Local>> for Rc<Local>`
//! through), but that lust's lang facts do not model for selection. A
//! user impl that overlaps one conflicts with std (rustc E0119), and the
//! where-clause disproof of the overlap check counts them as impls.

use crate::check::collect::Program;
use crate::def::DefId;
use crate::diag::Span;
use crate::traits::index::{ImplRecord, Synthetic, pattern_head};
use crate::ty::*;

/// Builds the shadow impls into the index; `finish_collect` calls it
/// before coherence. Each entry is written as rustc's std has it.
pub(crate) fn add_std_shadow_impls(p: &mut Program) {
    let lang = p.gx.lang;
    let t = p.tys.intern(TyKind::Param(ParamId(0)));
    let u = p.tys.intern(TyKind::Param(ParamId(1)));
    let (rt, mt) = (p.tys.mk_ref(false, t), p.tys.mk_ref(true, t));
    let (ru, mu) = (p.tys.mk_ref(false, u), p.tys.mk_ref(true, u));
    let (bt, rct) = (p.tys.mk_boxed(t), p.tys.mk_rc(t));
    // (trait, Self, inputs, params, where clauses as (trait, Self, inputs)).
    type Clause = (DefId, Ty, Vec<Ty>);
    let mut table: Vec<(DefId, Ty, Vec<Ty>, u32, Vec<Clause>)> = Vec::new();
    for r in [rt, mt] {
        for tr in [lang.debug, lang.display, lang.hash, lang.eq, lang.ord] {
            table.push((tr, r, vec![], 1, vec![(tr, t, vec![])]));
        }
        // `AsRef<U> for &T` and `&mut T` where `T: AsRef<U>`.
        table.push((lang.as_ref, r, vec![u], 2, vec![(lang.as_ref, t, vec![u])]));
        table.push((lang.borrow, r, vec![t], 1, vec![]));
    }
    // `PartialEq<&B> for &A` (and the `&mut` mixes), `PartialOrd` likewise.
    for (a, b) in [(rt, ru), (rt, mu), (mt, ru), (mt, mu)] {
        table.push((lang.partial_eq, a, vec![b], 2, vec![(lang.partial_eq, t, vec![u])]));
    }
    for (a, b) in [(rt, ru), (mt, mu)] {
        table.push((lang.partial_ord, a, vec![b], 2, vec![(lang.partial_ord, t, vec![u])]));
    }
    for tr in [lang.iterator, lang.double_ended_iterator, lang.exact_size_iterator, lang.hasher] {
        table.push((tr, mt, vec![], 1, vec![(tr, t, vec![])]));
        table.push((tr, bt, vec![], 1, vec![(tr, t, vec![])]));
    }
    table.push((lang.as_mut, mt, vec![u], 2, vec![(lang.as_mut, t, vec![u])]));
    table.push((lang.borrow_mut, mt, vec![t], 1, vec![]));
    // core's `impl<T: ?Sized> !Clone for &mut T`: a user `Clone` for
    // `&mut Local` conflicts with it (rustc E0751).
    table.push((lang.clone, mt, vec![], 1, vec![]));
    table.push((lang.display, bt, vec![], 1, vec![(lang.display, t, vec![])]));
    table.push((lang.display, rct, vec![], 1, vec![(lang.display, t, vec![])]));
    for w in [bt, rct] {
        table.push((lang.as_ref, w, vec![t], 1, vec![]));
        table.push((lang.borrow, w, vec![t], 1, vec![]));
        table.push((lang.drop, w, vec![], 1, vec![]));
    }
    table.push((lang.as_mut, bt, vec![t], 1, vec![]));
    table.push((lang.borrow_mut, bt, vec![t], 1, vec![]));
    table.push((lang.from, rct, vec![bt], 1, vec![]));
    let names = [p.interner.intern("T"), p.interner.intern("U")];
    for (i, (trait_def, self_pat, inputs, n, clauses)) in table.into_iter().enumerate() {
        let negative = trait_def == lang.clone && self_pat == mt;
        let trait_ref = mk_ref(p, trait_def, self_pat, &inputs);
        let predicates = clauses.into_iter().map(|(d, s, ins)| Predicate::Trait(Binder::plain(mk_ref(p, d, s, &ins)))).collect();
        let params = (0..n as usize).map(|k| ParamDef { name: names[k], kind: ParamKind::Type, span: Span::default() }).collect();
        let Some((head, blanket)) = pattern_head(&p.tys, self_pat, &[]) else { continue };
        let def = DefId {
            krate: p.krate,
            hash: crate::def::item_hash(p.stable_crate, &["<std-shadow>"], crate::def::DefKind::Impl, "impl", i as u32),
        };
        p.gx.index.insert_shadow(ImplRecord {
            def,
            trait_def,
            trait_ref,
            generics: Generics { parent: None, params, predicates },
            assoc_tys: Vec::new(),
            assoc_consts: Vec::new(),
            items: Vec::new(),
            synthetic: Synthetic::Lang,
            local: false,
            span: Span::default(),
            head,
            blanket,
        }, negative);
    }
}

fn mk_ref(p: &mut Program, def: DefId, self_ty: Ty, inputs: &[Ty]) -> TraitRef {
    let mut a = vec![GenericArg::Ty(self_ty)];
    a.extend(inputs.iter().map(|&x| GenericArg::Ty(x)));
    let args = p.tys.intern_args(&a);
    TraitRef { def, args, bindings: EMPTY_BINDINGS }
}
