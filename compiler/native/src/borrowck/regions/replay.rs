// Regions of impl selection (tf-borrow round 9).
//
// Check proves trait bounds with regions erased. A bound `Q: Tk<A>` that
// selects `impl Tk<&'static u32> for S` makes `A`'s region `'static`, and
// one that selects `impl<'x> Tk<&'x u32, &'x u32> for S` makes the two
// regions equal (trait args are invariant; rustc E0597, E0521). At a call
// that pairs the callee's signature, `replay_bounds` takes each regioned
// bound of the callee (`RegionSig::bounds`: the fn's, its impl's, its
// trait's, and a trait method's implicit `Self: Tr<P..>`), finds the one
// impl whose header matches the call's types, and adds the header's
// regions, the impl's lifetime facts, its `T: 'a` clauses, its associated
// type values (for `Q: Tk<Out = A>`), and its own trait where-clauses,
// replayed the same way to a bounded depth. A Self type that is a type
// param of this body replays this body's own bound instead. A bound it
// cannot pair, or that two impls may match, adds nothing.
use super::*;
use std::rc::Rc;

/// The types and regions a regioned type is read under: a callee's type
/// params and universals at a call, or an impl's params after its header
/// matched.
#[derive(Default)]
struct Frame {
    tys: FxMap<u32, Act>,
    lts: FxMap<u32, u32>,
}

/// A type at a replay: regioned under a frame, erased with its slot vars,
/// or free (a param nothing binds here: no region ties to it).
#[derive(Clone)]
enum Act {
    Reg(Ty, Rc<Frame>),
    Er(Ty, Vec<u32>),
    Free,
}

/// The shape of an `Act`, one level deep. `Other` is a type the replay
/// does not look into (a type param of this body, a projection, a dyn, a
/// fn pointer, a closure): it may equal anything.
enum View {
    Ref(Option<u32>, Mut, Act),
    Adt(DefId, Vec<Option<u32>>, Vec<Act>),
    Tuple(Vec<Act>),
    Elem(u8, Act),
    Scalar(Ty),
    Other(Option<Ty>),
}

/// How an impl header matches a bound's types.
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

/// What one header match found: the impl's params, its lifetimes' vars,
/// and the region requirements the match adds when it is the one.
#[derive(Default)]
struct Match {
    tys: FxMap<u32, Act>,
    lts: FxMap<u32, u32>,
    ties: Vec<(u32, u32)>,
    statics: Vec<u32>,
}

/// Bounds on the work of one call's replay (nested where-clauses).
const MAX_DEPTH: u32 = 6;
const MAX_STEPS: u32 = 256;

/// True when `t` names `'static` or a lifetime param.
fn names_region(tys: &impl TyIntern, t: Ty) -> bool {
    let mut rs = Vec::new();
    regions_of(tys, t, &mut rs);
    rs.iter().any(|r| matches!(r, Region::Static | Region::Param(_)))
}

fn is_scalar(k: &TyKind) -> bool {
    matches!(k, TyKind::Unit | TyKind::Int(_) | TyKind::Float(_) | TyKind::Bool | TyKind::Char | TyKind::Str | TyKind::Never)
}

impl ImplRegions {
    /// True when a bound that selects the impl may learn something about
    /// regions from it (collect keeps only those).
    pub fn wanted(&self, tys: &impl TyIntern) -> bool {
        !self.facts.is_empty()
            || !self.preds.is_empty()
            || self.header.iter().any(|&t| names_region(tys, t))
            || self.assoc.iter().any(|&(_, t)| names_region(tys, t))
    }

    /// True when the impl itself ties regions: `'static` or a lifetime
    /// named twice in its header, lifetime facts, outlives clauses, or
    /// regions in an associated type value.
    fn ties_regions(&self, tys: &impl TyIntern) -> bool {
        let mut rs = Vec::new();
        for &t in &self.header {
            regions_of(tys, t, &mut rs);
        }
        let named: Vec<u32> = rs.iter().filter_map(|r| if let Region::Param(k) = r { Some(*k) } else { None }).collect();
        rs.contains(&Region::Static)
            || named.iter().enumerate().any(|(i, k)| named[..i].contains(k))
            || !self.facts.is_empty()
            || self.preds.iter().any(|pr| matches!(pr, Predicate::TypeOutlives(..) | Predicate::Outlives(..)))
            || self.assoc.iter().any(|&(_, t)| names_region(tys, t))
    }
}

impl RegionTables {
    /// The traits some impl of which ties regions, itself or through its
    /// trait where-clauses (a fixpoint), built on first use.
    pub(super) fn replay_traits(&self, p: &Program) -> &FxSet<DefId> {
        self.replay_traits.get_or_init(|| {
            let mut out: FxSet<DefId> = FxSet::default();
            let impls: Vec<(DefId, &ImplRegions)> =
                p.gx.index.all_impls().filter_map(|(_, rec)| self.impl_regions.get(&rec.def).map(|ir| (rec.trait_def, ir))).collect();
            for &(d, ir) in &impls {
                if ir.ties_regions(&p.tys) {
                    out.insert(d);
                }
            }
            loop {
                let mut grew = false;
                for &(d, ir) in &impls {
                    if out.contains(&d) {
                        continue;
                    }
                    let nested = ir.preds.iter().any(|pr| match pr {
                        Predicate::Trait(b) => {
                            out.contains(&b.value.def)
                                || p.tys
                                    .args(b.value.args)
                                    .iter()
                                    .chain(p.tys.bindings(b.value.bindings).iter().map(|(_, g)| g))
                                    .any(|g| g.as_ty().is_some_and(|t| names_region(&p.tys, t)))
                        }
                        Predicate::AssocEq(_) => true,
                        _ => false,
                    });
                    if nested {
                        out.insert(d);
                        grew = true;
                    }
                }
                if !grew {
                    break out;
                }
            }
        })
    }
}

impl Builder<'_, '_> {
    /// Replays the impls the callee's bounds select at a call (see the
    /// file comment). `univ` and `binds` are the call's instantiation.
    pub(super) fn replay_bounds(&mut self, sig: &RegionSig, univ: &[u32], binds: &FxMap<u32, Vec<u32>>) {
        if sig.bounds.is_empty() {
            return;
        }
        let p = self.p;
        let mut frame = Frame::default();
        for (&j, vs) in binds {
            if let Some(&t) = self.bind_tys.get(&j) {
                frame.tys.insert(j, Act::Er(t, vs.clone()));
            }
        }
        for (k, &v) in univ.iter().enumerate() {
            frame.lts.insert(k as u32, v);
        }
        let frame = Rc::new(frame);
        let mut steps = 0;
        for pred in &sig.bounds {
            match *pred {
                Predicate::Trait(b) => self.replay_pred(b, &frame, 0, &mut steps),
                Predicate::AssocEq(b) if b.count == 0 => {
                    let acts: Vec<Act> =
                        p.tys.args(b.value.trait_args).iter().filter_map(|g| g.as_ty()).map(|t| Act::Reg(t, frame.clone())).collect();
                    if let Some(tr) = p.gx.index.trait_of_assoc(b.value.assoc) {
                        self.replay_trait(tr, &acts, &[(b.value.assoc, Act::Reg(b.value.ty, frame.clone()))], 0, &mut steps);
                    }
                }
                _ => {}
            }
        }
    }

    /// An unsizing of `src` (erased, slots `data`) to `dyn_ty` (its trait
    /// args' slots `args`, empty when unknown): `src` implements the
    /// principal trait with those args (rustc E0597 when the impl it
    /// selects is for `B<'static>`).
    pub(super) fn replay_dyn(&mut self, src: Ty, data: &[u32], dyn_ty: Ty, args: &[u32]) {
        let tys = &self.body.tys;
        let TyKind::Dyn(b, _) = tys.kind(dyn_ty) else { return };
        let Some(pr) = tys.dyn_bounds(b).principal else { return };
        if pr.count != 0 || self.sc.slots(src) as usize != data.len() {
            return;
        }
        let es: Vec<Ty> = tys.args(pr.value.args).iter().skip(1).filter_map(|g| g.as_ty()).collect();
        let bs: Vec<(DefId, Ty)> = tys.bindings(pr.value.bindings).iter().filter_map(|&(d, g)| g.as_ty().map(|t| (d, t))).collect();
        let mut acts = vec![Act::Er(src, data.to_vec())];
        let mut binds = Vec::new();
        let mut at = 0usize;
        for e in es {
            acts.push(if args.is_empty() { Act::Free } else { self.er_part(args, &mut at, e) });
        }
        for (d, t) in bs {
            binds.push((d, if args.is_empty() { Act::Free } else { self.er_part(args, &mut at, t) }));
        }
        self.replay_trait(pr.value.def, &acts, &binds, 0, &mut 0);
    }

    /// A value of user Adt `dt` (slots `dv`) is built: its type is
    /// well-formed, so the Adt's own bounds hold for its args.
    pub(super) fn replay_adt(&mut self, dt: Ty, dv: &[u32]) {
        let p = self.p;
        let TyKind::Adt(def, _) = self.body.tys.kind(dt) else { return };
        let Some(preds) = p.regions.adt_bounds.get(&def) else { return };
        if self.sc.slots(dt) as usize != dv.len() {
            return;
        }
        let View::Adt(_, lts, parts) = self.view(Act::Er(dt, dv.to_vec())) else { return };
        let mut frame = Frame::default();
        for (i, v) in lts.into_iter().enumerate() {
            if let Some(v) = v {
                frame.lts.insert(i as u32, v);
            }
        }
        for (j, a) in parts.into_iter().enumerate() {
            frame.tys.insert(j as u32, a);
        }
        let frame = Rc::new(frame);
        let mut steps = 0;
        for pred in preds {
            if let Predicate::Trait(b) = *pred {
                self.replay_pred(b, &frame, 0, &mut steps);
            }
        }
    }

    /// A regioned trait bound (with its `Assoc = X` bindings) under `fr`.
    fn replay_pred(&mut self, b: Binder<TraitRef>, fr: &Rc<Frame>, depth: u32, steps: &mut u32) {
        let p = self.p;
        if b.count != 0 {
            return;
        }
        let acts: Vec<Act> = p.tys.args(b.value.args).iter().filter_map(|g| g.as_ty()).map(|t| Act::Reg(t, fr.clone())).collect();
        let binds: Vec<(DefId, Act)> =
            p.tys.bindings(b.value.bindings).iter().filter_map(|&(d, g)| g.as_ty().map(|t| (d, Act::Reg(t, fr.clone())))).collect();
        self.replay_trait(b.value.def, &acts, &binds, depth, steps);
    }

    /// `acts[0]: trait_def<acts[1..]>` holds, with `Assoc = X` bindings:
    /// add what the impl it selects (or this body's own bound) says about
    /// their regions. Fn-family bounds have their own rules.
    fn replay_trait(&mut self, trait_def: DefId, acts: &[Act], binds: &[(DefId, Act)], depth: u32, steps: &mut u32) {
        let p = self.p;
        if p.gx.lang.is_fn_family(trait_def) {
            return;
        }
        let Some(fr) = self.select_impl(trait_def, acts, depth, steps) else { return };
        let ir = p.regions.impl_regions.get(&fr.0);
        for (assoc, want) in binds {
            let name =
                p.gx.index.trait_record(trait_def).and_then(|r| r.assoc_items.iter().find(|&&(d, _, _)| d == *assoc).map(|&(_, n, _)| n));
            if let Some(&(_, t)) = ir.and_then(|ir| ir.assoc.iter().find(|&&(n, _)| Some(n) == name)) {
                let mut m = Match::default();
                if self.equate(Act::Reg(t, fr.1.clone()), want.clone(), &mut m) != Fit::No {
                    self.apply(&m);
                }
            }
        }
        self.replay_impl(&fr, depth, steps);
    }

    /// The one impl of `trait_def` whose header matches `acts`, with its
    /// header's region requirements added, and the frame its where-clauses
    /// read. None when this body's own bound proves it (replayed here),
    /// when nothing ties regions, or when no single impl surely matches.
    fn select_impl(&mut self, trait_def: DefId, acts: &[Act], depth: u32, steps: &mut u32) -> Option<(DefId, Rc<Frame>)> {
        let p = self.p;
        *steps += 1;
        if depth > MAX_DEPTH || *steps > MAX_STEPS || acts.is_empty() {
            return None;
        }
        let self_act = self.norm(acts[0].clone());
        if let Act::Er(t, _) = &self_act
            && let TyKind::Param(j) = self.body.tys.kind(*t)
        {
            self.replay_env(trait_def, j.0, acts);
            return None;
        }
        if matches!(self_act, Act::Free) || !p.regions.replay_traits(p).contains(&trait_def) {
            return None;
        }
        let mut found: Option<(DefId, Match)> = None;
        // A derived record that cannot match decides nothing: only one
        // that does (`get`) reads its Adt's derive list (`peek`).
        for &id in p.gx.index.impls_of_trait(trait_def) {
            let rec = p.gx.index.peek(id);
            let pats: Vec<Ty> = match p.regions.impl_regions.get(&rec.def) {
                Some(ir) => ir.header.clone(),
                None => p.tys.args(rec.trait_ref.args).iter().filter_map(|g| g.as_ty()).collect(),
            };
            if pats.len() != acts.len() {
                p.gx.index.get(id);
                return None;
            }
            let mut m = Match::default();
            let mut fit = Fit::Yes;
            for (&pat, a) in pats.iter().zip(acts) {
                fit = fit.and(self.match_ty(pat, a.clone(), &mut m));
                if fit == Fit::No {
                    break;
                }
            }
            if fit != Fit::No {
                p.gx.index.get(id);
            }
            match fit {
                Fit::No => {}
                Fit::Maybe => return None,
                Fit::Yes if found.is_some() => return None,
                Fit::Yes => found = Some((rec.def, m)),
            }
        }
        let (def, m) = found?;
        self.apply(&m);
        Some((def, Rc::new(Frame { tys: m.tys, lts: m.lts })))
    }

    /// The impl's lifetime facts, outlives clauses, and trait where-clauses
    /// under the frame its header match built.
    fn replay_impl(&mut self, (def, fr): &(DefId, Rc<Frame>), depth: u32, steps: &mut u32) {
        let p = self.p;
        let Some(ir) = p.regions.impl_regions.get(def) else { return };
        for &(a, b) in &ir.facts {
            let x = fr.lts.get(&a).copied();
            let y = if b == crate::check::generics::STATIC_FACT { Some(0) } else { fr.lts.get(&b).copied() };
            if let (Some(x), Some(y)) = (x, y) {
                self.edge(x, y);
            }
        }
        for pred in &ir.preds {
            match *pred {
                Predicate::TypeOutlives(t, r) => {
                    if let Some(rv) = region_var(r, fr)
                        && let Some(vs) = self.act_vars(Act::Reg(t, fr.clone()))
                    {
                        for v in vs {
                            self.edge(v, rv);
                        }
                    }
                }
                Predicate::Outlives(a, b) => {
                    if let (Some(x), Some(y)) = (region_var(a, fr), region_var(b, fr)) {
                        self.edge(x, y);
                    }
                }
                Predicate::Trait(b) => self.replay_pred(b, fr, depth + 1, steps),
                Predicate::AssocEq(b) if b.count == 0 => {
                    let acts: Vec<Act> =
                        p.tys.args(b.value.trait_args).iter().filter_map(|g| g.as_ty()).map(|t| Act::Reg(t, fr.clone())).collect();
                    if let Some(tr) = p.gx.index.trait_of_assoc(b.value.assoc) {
                        self.replay_trait(tr, &acts, &[(b.value.assoc, Act::Reg(b.value.ty, fr.clone()))], depth + 1, steps);
                    }
                }
                _ => {}
            }
        }
    }

    /// `P: trait_def<acts[1..]>` for this body's type param `P`: its one
    /// own bound on `trait_def` gives the args' regions (this body's
    /// universal `'a` is var `a + 1`). A closure body has none of its own.
    fn replay_env(&mut self, trait_def: DefId, j: u32, acts: &[Act]) {
        let p = self.p;
        if self.own.closure {
            return;
        }
        let Some(sig) = p.regions.body_sig(p, self.body.fn_id) else { return };
        let mut hits = sig.bounds.iter().filter_map(|pr| match pr {
            Predicate::Trait(b)
                if b.count == 0
                    && b.value.def == trait_def
                    && p.tys
                        .args(b.value.args)
                        .first()
                        .and_then(|g| g.as_ty())
                        .is_some_and(|t| p.tys.kind(t) == TyKind::Param(crate::ty::ParamId(j))) =>
            {
                Some(b.value.args)
            }
            _ => None,
        });
        let (Some(args), None) = (hits.next(), hits.next()) else { return };
        let own: Vec<Ty> = p.tys.args(args).iter().filter_map(|g| g.as_ty()).collect();
        if own.len() != acts.len() || !own.iter().skip(1).any(|&t| names_region(&p.tys, t)) {
            return;
        }
        let mut frame = Frame::default();
        for k in 0..sig.n_univ {
            frame.lts.insert(k, k + 1);
        }
        let frame = Rc::new(frame);
        let mut m = Match::default();
        for (&t, a) in own.iter().zip(acts).skip(1) {
            if self.equate(Act::Reg(t, frame.clone()), a.clone(), &mut m) != Fit::Yes {
                return;
            }
        }
        self.apply(&m);
    }

    fn apply(&mut self, m: &Match) {
        for &v in &m.statics {
            self.edge(v, 0);
        }
        for &(x, y) in &m.ties {
            self.edge(x, y);
            self.edge(y, x);
        }
    }

    /// Follows a regioned type param to what its frame binds it to. An
    /// error or inference type (a constant argument) is free.
    fn norm(&self, mut a: Act) -> Act {
        loop {
            match &a {
                Act::Reg(t, f) => match self.body.tys.kind(*t) {
                    TyKind::Param(j) => a = f.tys.get(&j.0).cloned().unwrap_or(Act::Free),
                    _ => return a,
                },
                Act::Er(t, _) if matches!(self.body.tys.kind(*t), TyKind::Error | TyKind::Infer(_)) => return Act::Free,
                _ => return a,
            }
        }
    }

    /// One level of `a`'s shape (see `View`).
    fn view(&mut self, a: Act) -> View {
        let tys = &self.body.tys;
        match self.norm(a) {
            Act::Free => View::Other(None),
            Act::Er(t, vars) => {
                let mut at = 0usize;
                match tys.kind(t) {
                    TyKind::Ref(r, m, inner) => {
                        let v = if bound(r) { None } else { vars.first().copied() };
                        at = usize::from(v.is_some());
                        View::Ref(v, m, self.er_part(&vars, &mut at, inner))
                    }
                    TyKind::Adt(d, args) => {
                        let n = adt_regions(self.p, &self.body.tys, t) as usize;
                        let lts: Vec<Option<u32>> = (0..n).map(|i| vars.get(i).copied()).collect();
                        at = n;
                        let es: Vec<Ty> = self.body.tys.args(args).iter().filter_map(|g| g.as_ty()).collect();
                        let parts = es.into_iter().map(|e| self.er_part(&vars, &mut at, e)).collect();
                        View::Adt(d, lts, parts)
                    }
                    TyKind::Tuple(ts) => {
                        let es = self.body.tys.tys_of(ts).to_vec();
                        View::Tuple(es.into_iter().map(|e| self.er_part(&vars, &mut at, e)).collect())
                    }
                    TyKind::Array(e, _) => View::Elem(0, self.er_part(&vars, &mut at, e)),
                    TyKind::Slice(e) => View::Elem(1, self.er_part(&vars, &mut at, e)),
                    TyKind::RawPtr(m, e) => View::Elem(2 + m as u8, self.er_part(&vars, &mut at, e)),
                    k if is_scalar(&k) => View::Scalar(t),
                    _ => View::Other(Some(t)),
                }
            }
            Act::Reg(t, f) => match tys.kind(t) {
                TyKind::Ref(r, m, inner) => View::Ref(if bound(r) { None } else { region_var(r, &f) }, m, Act::Reg(inner, f)),
                TyKind::Adt(d, args) => {
                    let gs = tys.args(args).to_vec();
                    let lts = gs
                        .iter()
                        .filter_map(|g| {
                            if let GenericArg::Lifetime(r) = g
                                && *r != crate::ty::DYN_LATE
                            {
                                Some(region_var(*r, &f))
                            } else {
                                None
                            }
                        })
                        .collect();
                    let parts = gs.iter().filter_map(|g| g.as_ty()).map(|e| Act::Reg(e, f.clone())).collect();
                    View::Adt(d, lts, parts)
                }
                TyKind::Tuple(ts) => View::Tuple(tys.tys_of(ts).iter().map(|&e| Act::Reg(e, f.clone())).collect()),
                TyKind::Array(e, _) => View::Elem(0, Act::Reg(e, f)),
                TyKind::Slice(e) => View::Elem(1, Act::Reg(e, f)),
                TyKind::RawPtr(m, e) => View::Elem(2 + m as u8, Act::Reg(e, f)),
                k if is_scalar(&k) => View::Scalar(t),
                _ => View::Other(None),
            },
        }
    }

    /// The part of an erased type at slot `*at` of `vars` whose type is `e`.
    fn er_part(&mut self, vars: &[u32], at: &mut usize, e: Ty) -> Act {
        let n = self.sc.slots(e) as usize;
        let part = Act::Er(e, vars.get(*at..*at + n).map(|v| v.to_vec()).unwrap_or_default());
        *at += n;
        part
    }

    /// Matches impl header type `pat` (regioned, over the impl's params)
    /// against `a`, recording the impl's params and lifetimes in `m`.
    fn match_ty(&mut self, pat: Ty, a: Act, m: &mut Match) -> Fit {
        let tys = &self.body.tys;
        let pk = tys.kind(pat);
        if let TyKind::Param(k) = pk {
            return match m.tys.get(&k.0).cloned() {
                Some(prev) => self.equate(prev, a, m),
                None => {
                    m.tys.insert(k.0, a);
                    Fit::Yes
                }
            };
        }
        let a = self.norm(a);
        if matches!(a, Act::Free) {
            return Fit::Yes;
        }
        let view = self.view(a);
        let tys = &self.body.tys;
        match (pk, view) {
            (_, View::Other(t)) => {
                // An erased type the replay does not open: equal when the
                // header names the same type and no region.
                match t {
                    Some(t) if t == pat && !names_region(tys, pat) && !tys.flags(pat).contains_param() => Fit::Yes,
                    _ => Fit::Maybe,
                }
            }
            (TyKind::Ref(r, pm, pi), View::Ref(v, am, inner)) => {
                if pm != am {
                    return Fit::No;
                }
                region_at(r, v, m);
                self.match_ty(pi, inner, m)
            }
            (TyKind::Adt(pd, pa), View::Adt(ad, lts, parts)) => {
                if pd != ad {
                    return Fit::No;
                }
                let gs = tys.args(pa).to_vec();
                let plts: Vec<Region> = gs
                    .iter()
                    .filter_map(|g| {
                        if let GenericArg::Lifetime(r) = g
                            && *r != crate::ty::DYN_LATE
                        {
                            Some(*r)
                        } else {
                            None
                        }
                    })
                    .collect();
                let pts: Vec<Ty> = gs.iter().filter_map(|g| g.as_ty()).collect();
                if pts.len() != parts.len() {
                    return Fit::Maybe;
                }
                let mut fit = Fit::Yes;
                for (pt, part) in pts.into_iter().zip(parts) {
                    fit = fit.and(self.match_ty(pt, part, m));
                    if fit == Fit::No {
                        return fit;
                    }
                }
                if plts.len() != lts.len() {
                    // Hidden or erased lifetime args: nothing to pair.
                    return fit.and(if plts.iter().any(|r| matches!(r, Region::Static | Region::Param(_))) {
                        Fit::Maybe
                    } else {
                        Fit::Yes
                    });
                }
                for (r, v) in plts.into_iter().zip(lts) {
                    region_at(r, v, m);
                }
                fit
            }
            (TyKind::Tuple(ps), View::Tuple(parts)) => {
                let ps = tys.tys_of(ps).to_vec();
                if ps.len() != parts.len() {
                    return Fit::No;
                }
                let mut fit = Fit::Yes;
                for (pt, part) in ps.into_iter().zip(parts) {
                    fit = fit.and(self.match_ty(pt, part, m));
                    if fit == Fit::No {
                        return fit;
                    }
                }
                fit
            }
            (TyKind::Array(pe, _), View::Elem(0, e)) | (TyKind::Slice(pe), View::Elem(1, e)) => self.match_ty(pe, e, m),
            (TyKind::RawPtr(pm, pe), View::Elem(k, e)) if k >= 2 => {
                if 2 + pm as u8 != k {
                    return Fit::No;
                }
                self.match_ty(pe, e, m)
            }
            (k, View::Scalar(t)) if is_scalar(&k) => {
                if t == pat {
                    Fit::Yes
                } else {
                    Fit::No
                }
            }
            // Two different rigid shapes, or a header type the replay does
            // not open (a dyn, a fn pointer) against a rigid one.
            (k, _)
                if matches!(
                    k,
                    TyKind::Ref(..) | TyKind::Adt(..) | TyKind::Tuple(_) | TyKind::Array(..) | TyKind::Slice(_) | TyKind::RawPtr(..)
                ) || is_scalar(&k) =>
            {
                Fit::No
            }
            _ => Fit::Maybe,
        }
    }

    /// `a` and `b` are the same type: their regions are equal.
    fn equate(&mut self, a: Act, b: Act, m: &mut Match) -> Fit {
        let (a, b) = (self.norm(a), self.norm(b));
        if matches!(a, Act::Free) || matches!(b, Act::Free) {
            return Fit::Yes;
        }
        if let (Act::Er(x, xs), Act::Er(y, ys)) = (&a, &b)
            && x == y
        {
            m.ties.extend(xs.iter().copied().zip(ys.iter().copied()));
            return Fit::Yes;
        }
        match (self.view(a), self.view(b)) {
            (View::Ref(x, xm, xi), View::Ref(y, ym, yi)) => {
                if xm != ym {
                    return Fit::No;
                }
                if let (Some(x), Some(y)) = (x, y) {
                    m.ties.push((x, y));
                }
                self.equate(xi, yi, m)
            }
            (View::Adt(xd, xl, xp), View::Adt(yd, yl, yp)) => {
                if xd != yd || xp.len() != yp.len() {
                    return Fit::No;
                }
                if xl.len() == yl.len() {
                    for (x, y) in xl.into_iter().zip(yl) {
                        if let (Some(x), Some(y)) = (x, y) {
                            m.ties.push((x, y));
                        }
                    }
                }
                let mut fit = Fit::Yes;
                for (x, y) in xp.into_iter().zip(yp) {
                    fit = fit.and(self.equate(x, y, m));
                }
                fit
            }
            (View::Tuple(xs), View::Tuple(ys)) => {
                if xs.len() != ys.len() {
                    return Fit::No;
                }
                let mut fit = Fit::Yes;
                for (x, y) in xs.into_iter().zip(ys) {
                    fit = fit.and(self.equate(x, y, m));
                }
                fit
            }
            (View::Elem(i, x), View::Elem(j, y)) => {
                if i != j {
                    return Fit::No;
                }
                self.equate(x, y, m)
            }
            (View::Scalar(x), View::Scalar(y)) => {
                if x == y {
                    Fit::Yes
                } else {
                    Fit::No
                }
            }
            (View::Other(_), _) | (_, View::Other(_)) => Fit::Maybe,
            _ => Fit::No,
        }
    }

    /// Every region var of `a`; None when a part is not opened.
    fn act_vars(&mut self, a: Act) -> Option<Vec<u32>> {
        let mut out = Vec::new();
        self.act_vars_into(a, &mut out).then_some(out)
    }

    fn act_vars_into(&mut self, a: Act, out: &mut Vec<u32>) -> bool {
        if let Act::Er(_, vs) = &a {
            out.extend(vs.iter().copied());
            return true;
        }
        match self.view(a) {
            View::Ref(v, _, inner) => {
                out.extend(v);
                self.act_vars_into(inner, out)
            }
            View::Adt(_, lts, parts) => {
                out.extend(lts.into_iter().flatten());
                parts.into_iter().all(|x| self.act_vars_into(x, out))
            }
            View::Tuple(parts) => parts.into_iter().all(|x| self.act_vars_into(x, out)),
            View::Elem(_, e) => self.act_vars_into(e, out),
            View::Scalar(_) => true,
            View::Other(None) => true,
            View::Other(Some(_)) => false,
        }
    }
}

/// The var of region `r` under frame `f`: `'static` is var 0, a lifetime
/// param the frame's; None for one that is free here.
fn region_var(r: Region, f: &Frame) -> Option<u32> {
    match r {
        Region::Static => Some(0),
        Region::Param(k) => f.lts.get(&k).copied(),
        _ => None,
    }
}

/// Header region `r` at a place whose var is `v`.
fn region_at(r: Region, v: Option<u32>, m: &mut Match) {
    let Some(v) = v else { return };
    match r {
        Region::Static => m.statics.push(v),
        Region::Param(k) => match m.lts.get(&k) {
            Some(&w) => m.ties.push((v, w)),
            None => {
                m.lts.insert(k, v);
            }
        },
        _ => {}
    }
}
