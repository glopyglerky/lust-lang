// Regions of `Self::X` from another impl (tf-v07-bck).
//
// A trait impl method's region signature reads its own impl's associated
// types with regions (collect's `assoc_regioned`). `Self::Target` in a
// `DerefMut` impl, or `Self::Item` in a `DoubleEndedIterator` impl, names
// the binding of a supertrait's impl instead, so it stayed a projection
// over the erased Self type. That projection pairs with no erased type,
// the return got fresh universals, and every body that returned a borrow
// of `self` was rejected (L0733), with false L0303/L0304 at its callers.
// `normalize_self_projections` selects the one impl of the projection's
// trait whose regioned header matches the method's regioned Self type, and
// puts that impl's associated type in, with the impl's lifetimes and type
// params read through the match. Anything it cannot match exactly keeps
// the projection (the old, conservative signature).
use super::*;

/// Rewrites the `<Self as Tr>::X` projections of fn `f`'s region
/// signature (see the file comment). Runs with the erased signature's
/// normalization (`normalize_fn_sig`), when the impl index is complete.
pub fn normalize_self_projections(p: &mut Program, f: LocalDefId) {
    let Some(fs) = p.gx.fn_scopes.get(f.0 as usize) else { return };
    let (Some(self_e), Some(self_r)) = (fs.scope.self_ty, fs.scope.self_ty_regioned) else { return };
    let Some(sig) = p.regions.fns.get(&f) else { return };
    let tys: Vec<Ty> = sig.params.iter().copied().chain([sig.ret]).collect();
    if !tys.iter().any(|&t| p.tys.flags(t).contains_projection()) {
        return;
    }
    let mut out = Vec::with_capacity(tys.len());
    for t in tys {
        out.push(rewrite(p, t, self_e, self_r));
    }
    let Some(sig) = p.regions.fns.get_mut(&f) else { return };
    sig.ret = out.pop().unwrap_or(sig.ret);
    sig.params = out;
}

/// `t` with each projection on the erased Self type `self_e` replaced by
/// its impl's binding at regioned Self `self_r`, where one is found.
fn rewrite(p: &mut Program, t: Ty, self_e: Ty, self_r: Ty) -> Ty {
    if !p.tys.flags(t).contains_projection() {
        return t;
    }
    match p.tys.kind(t) {
        TyKind::Projection(a, args) => {
            if p.tys.args(args).first().and_then(|g| g.as_ty()) != Some(self_e) {
                return t;
            }
            binding_at(p, a.0, args, self_r).unwrap_or(t)
        }
        TyKind::Ref(r, m, e) => {
            let e = rewrite(p, e, self_e, self_r);
            p.tys.intern(TyKind::Ref(r, m, e))
        }
        TyKind::RawPtr(m, e) => {
            let e = rewrite(p, e, self_e, self_r);
            p.tys.intern(TyKind::RawPtr(m, e))
        }
        TyKind::Slice(e) => {
            let e = rewrite(p, e, self_e, self_r);
            p.tys.intern(TyKind::Slice(e))
        }
        TyKind::Array(e, n) => {
            let e = rewrite(p, e, self_e, self_r);
            p.tys.intern(TyKind::Array(e, n))
        }
        TyKind::Tuple(ts) => {
            let elems = p.tys.tys_of(ts).to_vec();
            let out: Vec<Ty> = elems.into_iter().map(|e| rewrite(p, e, self_e, self_r)).collect();
            let ts = p.tys.intern_tys(&out);
            p.tys.intern(TyKind::Tuple(ts))
        }
        TyKind::Adt(d, a) => {
            let raw = p.tys.args(a).to_vec();
            let out: Vec<GenericArg> = raw
                .into_iter()
                .map(|g| match g {
                    GenericArg::Ty(e) => GenericArg::Ty(rewrite(p, e, self_e, self_r)),
                    other => other,
                })
                .collect();
            p.tys.mk_adt(d, &out)
        }
        _ => t,
    }
}

/// The binding of assoc `assoc` in the one impl of its trait that matches
/// regioned Self `self_r` (and the projection's other trait args, which
/// must name no region). None when no impl, or more than one, may match,
/// or the binding names a region or param the match did not bind.
fn binding_at(p: &mut Program, assoc: DefId, args: ArgsId, self_r: Ty) -> Option<Ty> {
    let trait_def = p.gx.index.trait_of_assoc(assoc)?;
    let name = p.gx.index.trait_record(trait_def)?.assoc_items.iter().find(|(d, _, _)| *d == assoc)?.1;
    let rest: Vec<Ty> = p.tys.args(args).iter().skip(1).filter_map(|g| g.as_ty()).collect();
    if p.tys.args(args).iter().any(|g| matches!(g, GenericArg::Lifetime(_))) {
        return None;
    }
    for &t in &rest {
        let mut rs = Vec::new();
        regions_of(&p.tys, t, &mut rs);
        if !rs.is_empty() {
            return None;
        }
    }
    let mut found: Option<(Ty, Bind)> = None;
    for &id in p.gx.index.impls_of_trait(trait_def) {
        let rec = p.gx.index.peek(id);
        let (pats, bound): (Vec<Ty>, Option<Ty>) = match p.regions.impl_regions.get(&rec.def) {
            Some(ir) => (ir.header.clone(), ir.assoc.iter().find(|(n, _)| *n == name).map(|&(_, t)| t)),
            None => (
                p.tys.args(rec.trait_ref.args).iter().filter_map(|g| g.as_ty()).collect(),
                rec.assoc_tys.iter().find(|(d, _)| *d == assoc).map(|&(_, t)| t),
            ),
        };
        if pats.len() != rest.len() + 1 {
            return None;
        }
        let mut m = Bind::default();
        let mut fit = match_ty(&p.tys, pats[0], self_r, &mut m);
        for (&pat, &t) in pats[1..].iter().zip(&rest) {
            fit = fit.and(match_ty(&p.tys, pat, t, &mut m));
        }
        match fit {
            Fit::No => continue,
            Fit::Maybe => return None,
            Fit::Yes if found.is_some() => return None,
            Fit::Yes => {
                p.gx.index.get(id);
                found = Some((bound?, m));
            }
        }
    }
    let (bound, m) = found?;
    subst_bound(&mut p.tys, bound, &m)
}

/// An impl's lifetimes and type params as one header match bound them.
#[derive(Default)]
struct Bind {
    lts: FxMap<u32, Region>,
    tys: FxMap<u32, Ty>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fit {
    Yes,
    No,
    Maybe,
}

impl Fit {
    fn and(self, o: Fit) -> Fit {
        match (self, o) {
            (Fit::No, _) | (_, Fit::No) => Fit::No,
            (Fit::Maybe, _) | (_, Fit::Maybe) => Fit::Maybe,
            _ => Fit::Yes,
        }
    }
}

/// Binds impl lifetime `pat` to region `r`. A lifetime the header names
/// twice, or `'static`, puts a need on the regions that this pass does
/// not prove: Maybe.
fn match_region(pat: Region, r: Region, m: &mut Bind) -> Fit {
    if !matches!(r, Region::Param(_) | Region::Static) {
        return Fit::Maybe;
    }
    match pat {
        Region::Param(k) => match m.lts.get(&k) {
            Some(&old) if old != r => Fit::Maybe,
            _ => {
                m.lts.insert(k, r);
                Fit::Yes
            }
        },
        _ => Fit::Maybe,
    }
}

/// Matches impl header type `pat` (the impl's params) with type `t` (the
/// method's). No when the shapes differ, Maybe for a part this does not
/// look into.
fn match_ty(tys: &impl TyIntern, pat: Ty, t: Ty, m: &mut Bind) -> Fit {
    match (tys.kind(pat), tys.kind(t)) {
        (TyKind::Param(j), _) => match m.tys.get(&j.0) {
            Some(&old) if old != t => Fit::Maybe,
            _ => {
                m.tys.insert(j.0, t);
                Fit::Yes
            }
        },
        (TyKind::Ref(pr, pm, pi), TyKind::Ref(r, tm, ti)) => {
            if pm != tm {
                return Fit::No;
            }
            match_region(pr, r, m).and(match_ty(tys, pi, ti, m))
        }
        (TyKind::RawPtr(pm, pi), TyKind::RawPtr(tm, ti)) => {
            if pm != tm {
                return Fit::No;
            }
            match_ty(tys, pi, ti, m)
        }
        (TyKind::Slice(pi), TyKind::Slice(ti)) => match_ty(tys, pi, ti, m),
        (TyKind::Array(pi, pn), TyKind::Array(ti, tn)) => {
            if pn != tn {
                return Fit::Maybe;
            }
            match_ty(tys, pi, ti, m)
        }
        (TyKind::Tuple(ps), TyKind::Tuple(ts)) => {
            let (ps, ts) = (tys.tys_of(ps).to_vec(), tys.tys_of(ts).to_vec());
            if ps.len() != ts.len() {
                return Fit::No;
            }
            ps.iter().zip(&ts).fold(Fit::Yes, |f, (&a, &b)| f.and(match_ty(tys, a, b, m)))
        }
        (TyKind::Adt(pd, pa), TyKind::Adt(td, ta)) => {
            if pd != td {
                return Fit::No;
            }
            let (pa, ta) = (tys.args(pa).to_vec(), tys.args(ta).to_vec());
            if pa.len() != ta.len() {
                return Fit::Maybe;
            }
            let mut fit = Fit::Yes;
            for (a, b) in pa.iter().zip(&ta) {
                fit = fit.and(match (*a, *b) {
                    (GenericArg::Ty(a), GenericArg::Ty(b)) => match_ty(tys, a, b, m),
                    (GenericArg::Lifetime(a), GenericArg::Lifetime(b)) => match_region(a, b, m),
                    (a, b) if a == b => Fit::Yes,
                    _ => Fit::Maybe,
                });
            }
            fit
        }
        (pk, tk) => match (shape_of(&pk), shape_of(&tk)) {
            _ if is_plain(&pk) && pat == t => Fit::Yes,
            (Some(a), Some(b)) if a != b || is_plain(&pk) => Fit::No,
            _ => Fit::Maybe,
        },
    }
}

fn is_plain(k: &TyKind) -> bool {
    matches!(k, TyKind::Unit | TyKind::Int(_) | TyKind::Float(_) | TyKind::Bool | TyKind::Char | TyKind::Str | TyKind::Never)
}

/// A coarse kind tag: two types of different tags never unify. None for
/// kinds that may stand for another type (params, projections, opaques,
/// inference vars, errors).
fn shape_of(k: &TyKind) -> Option<u8> {
    Some(match k {
        TyKind::Unit => 0,
        TyKind::Int(_) => 1,
        TyKind::Float(_) => 2,
        TyKind::Bool => 3,
        TyKind::Char => 4,
        TyKind::Str => 5,
        TyKind::Never => 6,
        TyKind::Ref(..) => 7,
        TyKind::RawPtr(..) => 8,
        TyKind::Slice(_) => 9,
        TyKind::Array(..) => 10,
        TyKind::Tuple(_) => 11,
        TyKind::Adt(..) => 12,
        TyKind::FnPtr(_) => 13,
        TyKind::FnDef(..) => 14,
        TyKind::Closure(..) => 15,
        TyKind::Dyn(..) => 16,
        _ => return None,
    })
}

/// The impl's binding `t` with the matched lifetimes and type params put
/// in. None when it names a region other than a bound lifetime or
/// `'static`, or a part this does not rewrite (a dyn, a fn pointer, a
/// projection) that holds params or regions.
fn subst_bound(tys: &mut impl TyIntern, t: Ty, m: &Bind) -> Option<Ty> {
    let region = |r: Region| match r {
        Region::Static => Some(Region::Static),
        Region::Param(k) => m.lts.get(&k).copied(),
        _ => None,
    };
    Some(match tys.kind(t) {
        TyKind::Param(j) => *m.tys.get(&j.0)?,
        TyKind::Ref(r, mu, e) => {
            let (r, e) = (region(r)?, subst_bound(tys, e, m)?);
            tys.intern(TyKind::Ref(r, mu, e))
        }
        TyKind::RawPtr(mu, e) => {
            let e = subst_bound(tys, e, m)?;
            tys.intern(TyKind::RawPtr(mu, e))
        }
        TyKind::Slice(e) => {
            let e = subst_bound(tys, e, m)?;
            tys.intern(TyKind::Slice(e))
        }
        TyKind::Array(e, n) => {
            let e = subst_bound(tys, e, m)?;
            tys.intern(TyKind::Array(e, n))
        }
        TyKind::Tuple(ts) => {
            let elems = tys.tys_of(ts).to_vec();
            let out = elems.into_iter().map(|e| subst_bound(tys, e, m)).collect::<Option<Vec<Ty>>>()?;
            let ts = tys.intern_tys(&out);
            tys.intern(TyKind::Tuple(ts))
        }
        TyKind::Adt(d, a) => {
            let raw = tys.args(a).to_vec();
            let mut out = Vec::with_capacity(raw.len());
            for g in raw {
                out.push(match g {
                    GenericArg::Ty(e) => GenericArg::Ty(subst_bound(tys, e, m)?),
                    GenericArg::Lifetime(r) => GenericArg::Lifetime(region(r)?),
                    other => other,
                });
            }
            tys.mk_adt(d, &out)
        }
        k if is_plain(&k) => t,
        _ => {
            let mut rs = Vec::new();
            regions_of(tys, t, &mut rs);
            if !rs.is_empty() || tys.flags(t).contains_param() || tys.flags(t).contains_projection() {
                return None;
            }
            t
        }
    })
}
