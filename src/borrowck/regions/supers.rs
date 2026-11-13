// Regions of an impl's trait obligations (tf-borrow round 10).
//
// `impl<'a> Sup<&'a u32> for S {}` with `trait Sup<A>: Tk<A>` owes
// `S: Tk<&'a u32>` for every `'a`. Check proves it with regions erased, so
// `impl Tk<&'static u32> for S` passed; rustc rejects the impl (E0308,
// lifetime mismatch). `check_impl_supers` takes each user trait impl and
// each regioned bound of its trait (`RegionTables::trait_preds`:
// supertraits and where-clauses), puts the impl's header in, finds the one
// impl that matches (as `replay.rs` does at calls), and collects what its
// header, its `'x: 'y` facts and its own where-clauses (to a bounded depth)
// need of the impl's lifetimes. Each need must follow from the impl's
// declared facts and the implied bounds of its header. A bound that two
// impls may match, a part lust does not look into (a type param of the
// impl, a dyn, a projection) adds nothing. A trait where-clause of the
// impl proves a bound only at the arguments it writes, and only when
// the region equalities that takes follow from the impl's facts
// (rustc's rule, over a probed matrix): `S: Tag<&'static u8>` does not
// prove the `S: Tag<&'a u8>` a supertrait owes, a clause at another
// shape (`Tk<&'a u32>` for a `Tk<(&'a u32, &'a u32)>` goal) proves
// nothing, a clause that does not prove leaves the bound to the one
// matching impl, and two clauses that could prove it at different
// lifetimes make the proof ambiguous (rustc E0283).
use super::*;
use std::rc::Rc;

/// A region of the checked impl: `'static`, its lifetime param `k`, or one
/// nothing here names (no need on it).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Re {
    Static,
    Univ(u32),
    Free,
}

/// The types and regions a regioned type is read under: the checked
/// impl's own (`None`), or a trait's or a selected impl's params.
#[derive(Default)]
struct Frame {
    tys: FxMap<u32, Val>,
    lts: FxMap<u32, Re>,
}

#[derive(Clone)]
struct Val(Ty, Option<Rc<Frame>>);

/// One header match: the selected impl's params and the needs `x: y`.
#[derive(Default)]
struct Match {
    tys: FxMap<u32, Val>,
    lts: FxMap<u32, Re>,
    needs: Vec<(Re, Re)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fit {
    Yes,
    No,
    Maybe,
}

/// What the checked impl's own where-clauses say about one
/// obligation (`Cx::clause_proof`).
enum ClauseProof {
    /// One clause proves it; these needs follow from the facts.
    Proven(Vec<(Re, Re)>),
    /// A proving clause and another candidate read regions
    /// differently: the proof is ambiguous (rustc E0283).
    Ambiguous,
    /// A part this walk cannot read: nothing learned.
    Unreadable,
    /// No clause proves it. The needs of the first
    /// shape-matching clause, for when no impl matches either.
    NoProof(Vec<(Re, Re)>),
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

const MAX_DEPTH: u32 = 4;

struct Cx<'p> {
    p: &'p Program,
    /// The checked impl's own where-clauses and their supertraits,
    /// each read under its frame (`None`: the impl's own types).
    own: Vec<(TraitRef, Option<Rc<Frame>>)>,
    /// The checked impl's known `x: y` facts (`known_facts`).
    known: Vec<(Re, Re)>,
    /// A trait whose proof by the own clauses is ambiguous, once.
    ambig: Option<DefId>,
    /// `x: y` between the checked impl's regions.
    needs: Vec<(Re, Re)>,
    /// `T: y` for the checked impl's type param `T`.
    ty_needs: Vec<(u32, Re)>,
    steps: u32,
}

/// Checks the trait obligations of every user trait impl by regions (see
/// the file comment). Runs once collect has every impl and trait table.
pub fn check_impl_supers(p: &Program) -> Vec<Diag> {
    let mut errs = Vec::new();
    if p.regions.trait_preds.is_empty() {
        return errs;
    }
    for (_, rec) in p.gx.index.all_impls() {
        if !rec.local || rec.synthetic != crate::traits::index::Synthetic::None {
            continue;
        }
        let Some(preds) = p.regions.trait_preds.get(&rec.trait_def) else { continue };
        // An impl with no regions in its header, facts or where-clauses
        // (collect keeps none): its erased header, for `T: 'static` needs.
        let plain;
        let ir = match p.regions.impl_regions.get(&rec.def) {
            Some(ir) => ir,
            None => {
                let header = p.tys.args(rec.trait_ref.args).iter().filter_map(|g| g.as_ty()).collect();
                plain = ImplRegions { header, facts: Vec::new(), preds: Vec::new(), assoc: Vec::new() };
                &plain
            }
        };
        if let Some(d) = check_impl(p, rec, ir, preds) {
            errs.push(d);
        }
    }
    errs
}

fn check_impl(p: &Program, rec: &crate::traits::index::ImplRecord, ir: &ImplRegions, preds: &[Predicate]) -> Option<Diag> {
    let mut frame = Frame::default();
    for (j, &t) in ir.header.iter().enumerate() {
        frame.tys.insert(j as u32, Val(t, None));
    }
    if let Some(h) = p.regions.impl_heads.get(&rec.def) {
        for (k, &r) in h.trait_lifetimes.iter().enumerate() {
            frame.lts.insert(k as u32, outer_re(r));
        }
    }
    let frame = Rc::new(frame);
    // A trait where-clause of the impl may prove a bound on its trait
    // or a supertrait of it, but only at its written arguments. Keep
    // the clauses themselves; a supertrait is read in a frame that
    // puts the clause's arguments in the trait's slots (regioned
    // preds where collect kept them, the erased record otherwise).
    let mut own: Vec<(TraitRef, Option<Rc<Frame>>)> = ir
        .preds
        .iter()
        .filter_map(|q| if let Predicate::Trait(b) = q { (b.count == 0).then_some((b.value, None)) } else { None })
        .collect();
    let mut i = 0;
    while i < own.len() && own.len() < 128 {
        let (cr, cf) = own[i].clone();
        i += 1;
        let preds: Vec<Predicate> = match p.regions.trait_preds.get(&cr.def) {
            Some(ps) => ps.clone(),
            None => p.gx.index.trait_record(cr.def).map_or_else(Vec::new, |tr| tr.generics.predicates.clone()),
        };
        if preds.is_empty() {
            continue;
        }
        let mut f2 = Frame::default();
        for (j, t) in p.tys.args(cr.args).iter().filter_map(|g| g.as_ty()).enumerate() {
            f2.tys.insert(j as u32, Val(t, cf.clone()));
        }
        for (k, r) in p.tys.args(cr.args).iter().filter_map(|g| if let GenericArg::Lifetime(r) = g { Some(*r) } else { None }).enumerate() {
            f2.lts.insert(k as u32, re_in(r, &cf));
        }
        let f2 = Rc::new(f2);
        for q in preds {
            if let Predicate::Trait(b) = q
                && b.count == 0
                && !own.iter().any(|(o, of)| *o == b.value && of.as_ref().is_some_and(|x| Rc::ptr_eq(x, &f2)))
            {
                own.push((b.value, Some(f2.clone())));
            }
        }
    }
    let known = known_facts(p, ir);
    let mut cx = Cx { p, own, known, ambig: None, needs: Vec::new(), ty_needs: Vec::new(), steps: 0 };
    let name = |d: DefId| p.gx.index.trait_record(d).map_or_else(|| "?".to_string(), |r| p.display_path(r.name));
    let mut bad = None;
    for pred in preds {
        let Predicate::Trait(b) = *pred else { continue };
        if b.count != 0 || p.gx.lang.is_fn_family(b.value.def) {
            continue;
        }
        cx.needs.clear();
        cx.ty_needs.clear();
        cx.oblige(b.value, &frame, 0);
        if let Some(g) = cx.ambig {
            return Some(Diag::new(
                "L0610",
                rec.span,
                format!(
                    "this impl of `{}` has more than one where clause that can prove its `{g}` bound, at different lifetimes",
                    name(rec.trait_def),
                    g = name(g)
                ),
                "Remove the extra where clauses, or give them the same lifetimes.",
            ));
        }
        if let Some(&(_, y)) = cx.needs.iter().find(|&&(x, y)| !holds(&cx.known, x, y)) {
            bad = Some((b.value.def, "a lifetime in the impl's header", y));
            break;
        }
        let ty_known = known_ty_facts(p, ir);
        if let Some(&(_, y)) = cx.ty_needs.iter().find(|&&(j, y)| !ty_holds(&cx.known, &ty_known, j, y)) {
            bad = Some((b.value.def, "a type parameter of the impl", y));
            break;
        }
    }
    let (goal, what, y) = bad?;
    let life = if y == Re::Static { "`'static`" } else { "a lifetime of the impl" };
    Some(Diag::new(
        "L0617",
        rec.span,
        format!(
            "{what} may not live long enough: this impl of `{}` owes `{g}`, and the `{g}` impl it selects needs it to outlive {life}",
            name(rec.trait_def),
            g = name(goal)
        ),
        "Write the lifetimes the selected impl has (for example `'static`), or add the bound it needs (`'a: 'b`, `T: 'static`).",
    ))
}

/// The checked impl's known `x: y` facts: declared ones and the implied
/// bounds of its header types (`&'a &'b T`: `'b: 'a`; an Adt's declared
/// facts).
fn known_facts(p: &Program, ir: &ImplRegions) -> Vec<(Re, Re)> {
    let mut out: Vec<(Re, Re)> = ir
        .facts
        .iter()
        .map(|&(a, b)| (Re::Univ(a), if b == crate::check::generics::STATIC_FACT { Re::Static } else { Re::Univ(b) }))
        .collect();
    for &t in &ir.header {
        let mut fs = Vec::new();
        implied_regioned(&p.tys, t, &mut Vec::new(), &mut fs);
        adt_facts_in(p, t, &mut fs);
        out.extend(fs.into_iter().map(|(a, b)| (outer_re(a), outer_re(b))));
    }
    out
}

/// The declared `'x: 'y` facts of the user Adts in `t`, at their args.
fn adt_facts_in(p: &Program, t: Ty, out: &mut Vec<(Region, Region)>) {
    match p.tys.kind(t) {
        TyKind::Adt(def, args) => {
            let lts: Vec<Region> = p.tys.args(args).iter().filter_map(|g| if let GenericArg::Lifetime(r) = g { Some(*r) } else { None }).collect();
            for &(a, b) in p.regions.adt_facts.get(&def).map_or(&[][..], |v| v.as_slice()) {
                let y = if b == crate::check::generics::STATIC_FACT { Some(Region::Static) } else { lts.get(b as usize).copied() };
                if let (Some(&x), Some(y)) = (lts.get(a as usize), y) {
                    out.push((x, y));
                }
            }
            for g in p.tys.args(args) {
                if let GenericArg::Ty(e) = *g {
                    adt_facts_in(p, e, out);
                }
            }
        }
        TyKind::Ref(_, _, e) | TyKind::Array(e, _) | TyKind::Slice(e) | TyKind::RawPtr(_, e) => adt_facts_in(p, e, out),
        TyKind::Tuple(ts) => {
            for &e in p.tys.tys_of(ts) {
                adt_facts_in(p, e, out);
            }
        }
        _ => {}
    }
}

/// The checked impl's known `T: r` facts by type param: its `T: 'r`
/// where-clauses and the implied bounds of its header (`&'a T`: `T: 'a`).
fn known_ty_facts(p: &Program, ir: &ImplRegions) -> Vec<(u32, Re)> {
    let mut out: Vec<(u32, Re)> = ir
        .preds
        .iter()
        .filter_map(|q| match *q {
            Predicate::TypeOutlives(t, r) => match p.tys.kind(t) {
                TyKind::Param(j) => Some((j.0, outer_re(r))),
                _ => None,
            },
            _ => None,
        })
        .collect();
    for &t in &ir.header {
        params_under_refs(p, t, &mut Vec::new(), &mut out);
    }
    out
}

/// `T: 'r` for each type param `T` inside a reference `&'r ..` of `t`.
fn params_under_refs(p: &Program, t: Ty, outer: &mut Vec<Re>, out: &mut Vec<(u32, Re)>) {
    match p.tys.kind(t) {
        TyKind::Param(j) => out.extend(outer.iter().map(|&r| (j.0, r))),
        TyKind::Ref(r, _, e) => {
            outer.push(outer_re(r));
            params_under_refs(p, e, outer, out);
            outer.pop();
        }
        TyKind::Adt(_, args) => {
            for g in p.tys.args(args) {
                if let GenericArg::Ty(e) = *g {
                    params_under_refs(p, e, outer, out);
                }
            }
        }
        TyKind::Tuple(ts) => {
            for &e in p.tys.tys_of(ts) {
                params_under_refs(p, e, outer, out);
            }
        }
        TyKind::Array(e, _) | TyKind::Slice(e) => params_under_refs(p, e, outer, out),
        _ => {}
    }
}

/// True when the checked impl's type param `j` outlives `y` by `ty_known`
/// and the region facts.
fn ty_holds(known: &[(Re, Re)], ty_known: &[(u32, Re)], j: u32, y: Re) -> bool {
    y == Re::Free || ty_known.iter().any(|&(k, r)| k == j && holds(known, r, y))
}

fn outer_re(r: Region) -> Re {
    match r {
        Region::Static => Re::Static,
        Region::Param(k) => Re::Univ(k),
        _ => Re::Free,
    }
}

/// True when `x: y` follows from `known` (transitively; `'static`
/// outlives every region).
fn holds(known: &[(Re, Re)], x: Re, y: Re) -> bool {
    if x == y || x == Re::Static || x == Re::Free || y == Re::Free {
        return true;
    }
    let mut seen = vec![x];
    let mut i = 0;
    while i < seen.len() {
        let a = seen[i];
        i += 1;
        for &(u, v) in known {
            if u == a && !seen.contains(&v) {
                if v == y || v == Re::Static {
                    return true;
                }
                seen.push(v);
            }
        }
    }
    false
}

impl Cx<'_> {
    /// `args[0]: tr<args[1..]>` (read under `fr`) holds: add what the one
    /// impl that matches needs.
    fn oblige(&mut self, tr: TraitRef, fr: &Rc<Frame>, depth: u32) {
        let p = self.p;
        self.steps += 1;
        if depth > MAX_DEPTH || self.steps > 256 {
            return;
        }
        let fallback = match self.clause_proof(tr, fr) {
            ClauseProof::Proven(n) => {
                self.needs.extend(n);
                return;
            }
            ClauseProof::Ambiguous => {
                if self.ambig.is_none() {
                    self.ambig = Some(tr.def);
                }
                return;
            }
            ClauseProof::Unreadable => return,
            ClauseProof::NoProof(f) => f,
        };
        // A std trait none of whose impls ties regions: nothing to learn
        // (and its impl list is long).
        if !p.gx.index.trait_record(tr.def).is_some_and(|r| r.local) && !p.regions.replay_traits(p).contains(&tr.def) {
            return;
        }
        let args = p.tys.args(tr.args);
        // A trait with lifetime params: its impls' lifetime args are not
        // in the regioned headers.
        if args.iter().any(|g| !matches!(g, GenericArg::Ty(_))) {
            return;
        }
        let acts: Vec<Val> = args.iter().filter_map(|g| g.as_ty()).map(|t| Val(t, Some(fr.clone()))).collect();
        let mut found: Option<(DefId, Match)> = None;
        for &id in p.gx.index.impls_of_trait(tr.def) {
            let rec = p.gx.index.get(id);
            let pats: Vec<Ty> = match p.regions.impl_regions.get(&rec.def) {
                Some(ir) => ir.header.clone(),
                None => p.tys.args(rec.trait_ref.args).iter().filter_map(|g| g.as_ty()).collect(),
            };
            if pats.len() != acts.len() {
                return;
            }
            let mut m = Match::default();
            let mut fit = Fit::Yes;
            for (&pat, a) in pats.iter().zip(&acts) {
                fit = fit.and(self.match_ty(pat, a.clone(), &mut m));
                if fit == Fit::No {
                    break;
                }
            }
            match fit {
                Fit::No => {}
                Fit::Maybe => return,
                Fit::Yes if found.is_some() => return,
                Fit::Yes => found = Some((rec.def, m)),
            }
        }
        let Some((def, m)) = found else {
            // No impl matches either: a shape-matching clause is the
            // only proof left, and its needs do not hold.
            self.needs.extend(fallback);
            return;
        };
        self.needs.extend(m.needs.iter().copied());
        let Some(ir) = p.regions.impl_regions.get(&def) else { return };
        let lt = |k: u32| m.lts.get(&k).copied().unwrap_or(Re::Free);
        for &(a, b) in &ir.facts {
            let y = if b == crate::check::generics::STATIC_FACT { Re::Static } else { lt(b) };
            self.needs.push((lt(a), y));
        }
        let sel = Rc::new(Frame { tys: m.tys, lts: m.lts });
        for pred in &ir.preds {
            match *pred {
                Predicate::Outlives(a, b) => {
                    let (x, y) = (self.re(a, &Some(sel.clone())), self.re(b, &Some(sel.clone())));
                    self.needs.push((x, y));
                }
                Predicate::TypeOutlives(t, r) => {
                    let y = self.re(r, &Some(sel.clone()));
                    let (mut rs, mut ps) = (Vec::new(), Vec::new());
                    self.regions_in(Val(t, Some(sel.clone())), &mut rs, &mut ps, 0);
                    for x in rs {
                        self.needs.push((x, y));
                    }
                    for j in ps {
                        self.ty_needs.push((j, y));
                    }
                }
                Predicate::Trait(b) if b.count == 0 => self.oblige(b.value, &sel, depth + 1),
                _ => {}
            }
        }
    }

    /// The region `r` read under frame `fr` (`None`: the checked impl's).
    fn re(&self, r: Region, fr: &Option<Rc<Frame>>) -> Re {
        re_in(r, fr)
    }

    /// What the impl's own where-clauses say about the obligation
    /// `tr` (read under `fr`). A clause is a candidate only when
    /// every argument pair equates: a different shape (an element
    /// type for a pair goal) is no proof at all. A candidate
    /// proves the obligation when the region equalities that
    /// takes follow from the impl's facts; rustc accepts exactly
    /// the fact-equal cases over a probed matrix (an impl's
    /// `'a: 'b, 'b: 'a`, or `'a: 'static` against a `'static`
    /// goal). When one candidate proves it and another candidate
    /// reads its regions differently, the proof is ambiguous
    /// (rustc E0283), whether the other candidate proves it or
    /// not.
    ///
    /// A candidate that does not prove it is selected (its needs
    /// are the answer, no impl fallback) only when it mentions
    /// one of the impl's parameters, a type parameter in any
    /// argument (the Self type counts) or a lifetime parameter
    /// at any region slot. As probed, a clause at `'b` for a
    /// goal at `'a`, a clause at `'a` for a `'static` goal, and
    /// a clause at `'static` on a generic Self type are selected
    /// and fail; a clause at `'static` on a concrete Self type
    /// for a goal at `'a` mentions no parameter, is not
    /// assembled at all, and the one matching impl proves the
    /// bound instead. A candidate with a part this walk cannot
    /// read keeps the old rule: it proves, and adds nothing.
    fn clause_proof(&self, tr: TraitRef, fr: &Rc<Frame>) -> ClauseProof {
        let p = self.p;
        let oa = p.tys.args(tr.args).to_vec();
        // Per candidate: its needs, and whether it mentions an
        // impl parameter (eligible for selection).
        let mut yes: Vec<(Vec<(Re, Re)>, bool)> = Vec::new();
        let mut saw_maybe = false;
        for (cr, cf) in self.own.clone() {
            if cr.def != tr.def {
                continue;
            }
            let ca = p.tys.args(cr.args).to_vec();
            if ca.iter().chain(&oa).any(|g| !matches!(g, GenericArg::Ty(_))) {
                return ClauseProof::Unreadable;
            }
            if ca.len() != oa.len() {
                continue;
            }
            let mut m = Match::default();
            let mut fit = Fit::Yes;
            for (g, h) in ca.iter().zip(&oa) {
                let (Some(ct), Some(ot)) = (g.as_ty(), h.as_ty()) else { continue };
                fit = fit.and(self.equate(Val(ct, cf.clone()), Val(ot, Some(fr.clone())), &mut m));
            }
            match fit {
                Fit::Yes => {
                    // Eligible when the clause's arguments
                    // mention a type parameter or a lifetime
                    // parameter of the checked impl.
                    let mut mentions = false;
                    for g in &ca {
                        if let Some(ct) = g.as_ty() {
                            let (mut rs, mut ps) = (Vec::new(), Vec::new());
                            self.regions_in(Val(ct, cf.clone()), &mut rs, &mut ps, 0);
                            mentions |= !ps.is_empty() || rs.iter().any(|r| matches!(r, Re::Univ(_)));
                        }
                    }
                    yes.push((m.needs, mentions));
                }
                Fit::Maybe => saw_maybe = true,
                Fit::No => {}
            }
        }
        if saw_maybe {
            return ClauseProof::Unreadable;
        }
        // A candidate's region reading: its needs without the
        // pairs that always hold. Two clauses with the same
        // reading are one proof, not two.
        let key = |needs: &[(Re, Re)]| -> Vec<(Re, Re)> {
            needs.iter().copied().filter(|&(x, y)| x != y && x != Re::Free && y != Re::Free).collect()
        };
        let holds_all = |n: &[(Re, Re)]| n.iter().all(|&(x, y)| holds(&self.known, x, y));
        if let Some((n0, _)) = yes.iter().find(|(n, _)| holds_all(n)) {
            let k0 = key(n0);
            if yes.iter().any(|(n, _)| !same_needs(&key(n), &k0)) {
                return ClauseProof::Ambiguous;
            }
        }
        let chosen = yes.iter().find(|(n, e)| *e && holds_all(n)).or_else(|| yes.iter().find(|(_, e)| *e));
        match chosen {
            Some((n, _)) => ClauseProof::Proven(n.clone()),
            None => ClauseProof::NoProof(yes.into_iter().next().map_or_else(Vec::new, |(n, _)| n)),
        }
    }

    /// Follows a type param to what its frame binds it to. A param of the
    /// checked impl (no frame) stays; an unbound one is `None`.
    fn norm(&self, mut v: Val) -> Option<Val> {
        loop {
            let TyKind::Param(j) = self.p.tys.kind(v.0) else { return Some(v) };
            let Some(f) = &v.1 else { return Some(v) };
            v = f.tys.get(&j.0)?.clone();
        }
    }

    /// The checked impl's regions in `v`, and its type params there.
    fn regions_in(&self, v: Val, out: &mut Vec<Re>, params: &mut Vec<u32>, depth: u32) {
        let p = self.p;
        let Some(Val(t, fr)) = self.norm(v) else { return };
        if depth > 16 {
            return;
        }
        match p.tys.kind(t) {
            TyKind::Param(j) if fr.is_none() => params.push(j.0),
            TyKind::Ref(r, _, e) => {
                out.push(self.re(r, &fr));
                self.regions_in(Val(e, fr), out, params, depth + 1);
            }
            TyKind::Adt(_, args) => {
                for g in p.tys.args(args) {
                    match *g {
                        GenericArg::Lifetime(r) => out.push(self.re(r, &fr)),
                        GenericArg::Ty(e) => self.regions_in(Val(e, fr.clone()), out, params, depth + 1),
                        GenericArg::Const(_) => {}
                    }
                }
            }
            TyKind::Tuple(ts) => {
                for &e in p.tys.tys_of(ts) {
                    self.regions_in(Val(e, fr.clone()), out, params, depth + 1);
                }
            }
            TyKind::Array(e, _) | TyKind::Slice(e) | TyKind::RawPtr(_, e) => self.regions_in(Val(e, fr), out, params, depth + 1),
            _ => {}
        }
    }

    /// A selected impl's header region `pr` against the obligation's `o`:
    /// `'static` needs `o: 'static`; the impl's `'x` binds once, and a
    /// second use needs the two equal.
    fn match_re(&self, pr: Region, o: Re, m: &mut Match) {
        match pr {
            Region::Static => m.needs.push((o, Re::Static)),
            Region::Param(k) => match m.lts.get(&k) {
                Some(&x) => {
                    m.needs.push((x, o));
                    m.needs.push((o, x));
                }
                None => {
                    m.lts.insert(k, o);
                }
            },
            _ => {}
        }
    }

    /// Matches a selected impl's header type `pat` (its own params and
    /// lifetimes) against the obligation's type `o`.
    fn match_ty(&self, pat: Ty, o: Val, m: &mut Match) -> Fit {
        let p = self.p;
        if let TyKind::Param(j) = p.tys.kind(pat) {
            return match m.tys.get(&j.0).cloned() {
                Some(prev) => self.equate(prev, o, m),
                None => {
                    m.tys.insert(j.0, o);
                    Fit::Yes
                }
            };
        }
        let Some(Val(ot, ofr)) = self.norm(o) else { return Fit::Maybe };
        match (p.tys.kind(pat), p.tys.kind(ot)) {
            (TyKind::Ref(pr, pm, pi), TyKind::Ref(or, om, oi)) => {
                if pm != om {
                    return Fit::No;
                }
                let x = self.re(or, &ofr);
                let fit = self.match_ty(pi, Val(oi, ofr), m);
                if fit == Fit::Yes {
                    self.match_re(pr, x, m);
                }
                fit
            }
            (TyKind::Adt(pd, pa), TyKind::Adt(od, oa)) => {
                if pd != od {
                    return Fit::No;
                }
                let (pa, oa) = (p.tys.args(pa).to_vec(), p.tys.args(oa).to_vec());
                if pa.len() != oa.len() {
                    return Fit::Maybe;
                }
                let mut fit = Fit::Yes;
                let mut lts = Vec::new();
                for (g, h) in pa.iter().zip(&oa) {
                    match (*g, *h) {
                        (GenericArg::Ty(a), GenericArg::Ty(b)) => fit = fit.and(self.match_ty(a, Val(b, ofr.clone()), m)),
                        (GenericArg::Lifetime(a), GenericArg::Lifetime(b)) => lts.push((a, self.re(b, &ofr))),
                        _ => {}
                    }
                    if fit == Fit::No {
                        return fit;
                    }
                }
                if fit == Fit::Yes {
                    for (a, x) in lts {
                        self.match_re(a, x, m);
                    }
                }
                fit
            }
            (TyKind::Tuple(a), TyKind::Tuple(b)) => {
                let (a, b) = (p.tys.tys_of(a).to_vec(), p.tys.tys_of(b).to_vec());
                if a.len() != b.len() {
                    return Fit::No;
                }
                a.iter().zip(&b).fold(Fit::Yes, |f, (&x, &y)| if f == Fit::No { f } else { f.and(self.match_ty(x, Val(y, ofr.clone()), m)) })
            }
            (TyKind::Slice(a), TyKind::Slice(b)) | (TyKind::Array(a, _), TyKind::Array(b, _)) => self.match_ty(a, Val(b, ofr), m),
            // A param of the checked impl, a dyn, a fn pointer, a
            // projection, or a shape this walk does not read.
            (_, TyKind::Param(_) | TyKind::Projection(..) | TyKind::Dyn(..) | TyKind::FnPtr(_) | TyKind::Opaque(..) | TyKind::Infer(_) | TyKind::Error) => {
                Fit::Maybe
            }
            (TyKind::Dyn(..) | TyKind::FnPtr(_) | TyKind::Projection(..), _) => Fit::Maybe,
            (pk, ok) if scalar(&pk) || scalar(&ok) => {
                if pk == ok {
                    Fit::Yes
                } else {
                    Fit::No
                }
            }
            (pk, ok) if std::mem::discriminant(&pk) != std::mem::discriminant(&ok) => Fit::No,
            _ => Fit::Maybe,
        }
    }

    /// Two obligation-side types that one impl param binds twice, or
    /// a where-clause's argument against the goal's: their regions
    /// are equal, slot by slot. No when two concrete shapes cannot
    /// pair (an element type for a pair); Maybe when a part this
    /// walk does not read (a param, a projection, a dyn) keeps the
    /// shapes from pairing.
    fn equate(&self, a: Val, b: Val, m: &mut Match) -> Fit {
        let p = self.p;
        let (Some(Val(at, af)), Some(Val(bt, bf))) = (self.norm(a), self.norm(b)) else { return Fit::Maybe };
        match (p.tys.kind(at), p.tys.kind(bt)) {
            (TyKind::Ref(ar, am, ai), TyKind::Ref(br, bm, bi)) if am == bm => {
                let (x, y) = (self.re(ar, &af), self.re(br, &bf));
                m.needs.push((x, y));
                m.needs.push((y, x));
                self.equate(Val(ai, af), Val(bi, bf), m)
            }
            (TyKind::Adt(ad, aa), TyKind::Adt(bd, ba)) if ad == bd => {
                let (aa, ba) = (p.tys.args(aa).to_vec(), p.tys.args(ba).to_vec());
                let mut fit = Fit::Yes;
                for (g, h) in aa.iter().zip(&ba) {
                    match (*g, *h) {
                        (GenericArg::Ty(x), GenericArg::Ty(y)) => fit = fit.and(self.equate(Val(x, af.clone()), Val(y, bf.clone()), m)),
                        (GenericArg::Lifetime(x), GenericArg::Lifetime(y)) => {
                            let (x, y) = (self.re(x, &af), self.re(y, &bf));
                            m.needs.push((x, y));
                            m.needs.push((y, x));
                        }
                        _ => {}
                    }
                }
                fit
            }
            (TyKind::Tuple(x), TyKind::Tuple(y)) => {
                let (x, y) = (p.tys.tys_of(x).to_vec(), p.tys.tys_of(y).to_vec());
                if x.len() != y.len() {
                    return Fit::No;
                }
                x.iter().zip(&y).fold(Fit::Yes, |f, (&a, &b)| f.and(self.equate(Val(a, af.clone()), Val(b, bf.clone()), m)))
            }
            (TyKind::Slice(x), TyKind::Slice(y)) | (TyKind::Array(x, _), TyKind::Array(y, _)) => self.equate(Val(x, af), Val(y, bf), m),
            (TyKind::RawPtr(_, x), TyKind::RawPtr(_, y)) => self.equate(Val(x, af), Val(y, bf), m),
            (x, y) if scalar(&x) && x == y => Fit::Yes,
            (TyKind::Param(x), TyKind::Param(y)) if x == y && af.is_none() && bf.is_none() => Fit::Yes,
            // Concrete shapes that cannot pair: another Adt, the
            // other mutability, another scalar, another constructor.
            (TyKind::Adt(..), TyKind::Adt(..)) | (TyKind::Ref(..), TyKind::Ref(..)) => Fit::No,
            (x, y) if concrete(&x) && concrete(&y) => Fit::No,
            _ => Fit::Maybe,
        }
    }
}

/// The region `r` read under frame `fr` (`None`: the checked impl's).
fn re_in(r: Region, fr: &Option<Rc<Frame>>) -> Re {
    match (r, fr) {
        (Region::Static, _) => Re::Static,
        (Region::Param(k), None) => Re::Univ(k),
        (Region::Param(k), Some(f)) => f.lts.get(&k).copied().unwrap_or(Re::Free),
        _ => Re::Free,
    }
}

/// True when a type's outer shape is fixed here, so a different
/// concrete shape against it cannot pair in `equate`. A param, a
/// projection, a dyn, a fn type, or an inference hole can still
/// become the other side.
fn concrete(k: &TyKind) -> bool {
    !matches!(
        k,
        TyKind::Param(_)
            | TyKind::Projection(..)
            | TyKind::Dyn(..)
            | TyKind::FnPtr(_)
            | TyKind::FnDef(..)
            | TyKind::Closure(..)
            | TyKind::Opaque(..)
            | TyKind::Infer(_)
            | TyKind::Error
    )
}

/// Two need lists hold the same pairs, in any order.
fn same_needs(a: &[(Re, Re)], b: &[(Re, Re)]) -> bool {
    a.iter().all(|x| b.contains(x)) && b.iter().all(|x| a.contains(x))
}

fn scalar(k: &TyKind) -> bool {
    matches!(k, TyKind::Unit | TyKind::Int(_) | TyKind::Float(_) | TyKind::Bool | TyKind::Char | TyKind::Str | TyKind::Never)
}
