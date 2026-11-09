// Unused generic parameters of structs and enums (rustc E0392).
//
// rustc computes the variance of every parameter and rejects one that
// stays bivariant: a parameter no field type uses, or one that only
// appears as the argument of a user type at a position that is itself
// unused (a use only through recursion, `struct S<T>(Box<S<T>>)`, also
// through other types, `struct A<T>(Box<B<T>>); struct B<T>(Box<A<T>>);`).
// Every builtin and std type uses each of its parameters (references,
// raw and fn pointers, `dyn`, projections, `PhantomData`, `Vec`, ...), so
// only user Adt arguments need the fixpoint below. Const parameters are
// never checked, as in rustc.
use crate::ast::{self, ItemKind};
use crate::check::collect::Program;
use crate::diag::Diag;
use crate::ty::*;

/// Per Adt: which type params (by index) and lifetime params (by
/// position) a field uses.
struct Used {
    tys: Vec<bool>,
    lts: Vec<bool>,
}

/// Rejects every type or lifetime parameter of the given Adt templates
/// that no field type uses in a non-recursive way. `adts` are the
/// templates of this collect with their lifetime parameter spans.
pub fn check_unused_params(p: &Program, adts: &[(AdtId, Vec<crate::diag::Span>)], errs: &mut Vec<Diag>) {
    let fields: Vec<Vec<Ty>> = adts.iter().map(|&(id, _)| field_tys(p, id)).collect();
    let mut used: Vec<Used> = adts
        .iter()
        .zip(&fields)
        .map(|(&(id, _), fs)| {
            let def = p.gx.adt_info[id.0 as usize].0;
            let n_lts = p.gx.adt_lifetimes.get(&def).map_or(0, |v| v.len());
            // `PhantomData<T>` uses its `T` by definition (std source). An Adt with a field
            // type that did not resolve has its own error; it counts as using every
            // parameter, so its users get no follow-on "never used" error.
            let all = p.is_phantom_adt(id) || fs.contains(&ERROR);
            Used { tys: vec![all; p.gx.adt_generics[id.0 as usize].params.len()], lts: vec![fs.contains(&ERROR); n_lts] }
        })
        .collect();
    let slot: crate::intern::FxMap<AdtId, usize> = adts.iter().enumerate().map(|(i, &(id, _))| (id, i)).collect();
    // A monotone fixpoint: each sweep can only mark more uses.
    loop {
        let mut changed = false;
        for (i, fs) in fields.iter().enumerate() {
            for &t in fs {
                let mut mark = Mark { p, slot: &slot, used: &used, tys: Vec::new(), lts: Vec::new() };
                mark.walk(t);
                let (tys, lts) = (mark.tys, mark.lts);
                for k in tys {
                    if let Some(u) = used[i].tys.get_mut(k as usize)
                        && !*u
                    {
                        *u = true;
                        changed = true;
                    }
                }
                for k in lts {
                    if let Some(u) = used[i].lts.get_mut(k as usize)
                        && !*u
                    {
                        *u = true;
                        changed = true;
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    for (i, (id, spans)) in adts.iter().enumerate() {
        let id = *id;
        // A field type that did not resolve has its own error already.
        if fields[i].contains(&ERROR) {
            continue;
        }
        let generics = &p.gx.adt_generics[id.0 as usize];
        for (k, param) in generics.params.iter().enumerate() {
            if param.kind != ParamKind::Type || used[i].tys[k] {
                continue;
            }
            let name = p.interner.get(param.name);
            let recursive = fields[i].iter().any(|&t| {
                let mut all = Mark { p, slot: &slot, used: &[], tys: Vec::new(), lts: Vec::new() };
                all.walk(t);
                all.tys.contains(&(k as u32))
            });
            errs.push(if recursive {
                Diag::new(
                    "L0110",
                    param.span,
                    format!("type parameter `{name}` is only used recursively"),
                    "Use the parameter in a field outside the recursion, use `PhantomData`, or remove it.",
                )
            } else {
                Diag::new(
                    "L0110",
                    param.span,
                    format!("type parameter `{name}` is never used"),
                    "Use the parameter in a field type, or remove it.",
                )
            });
        }
        let def = p.gx.adt_info[id.0 as usize].0;
        let Some(names) = p.gx.adt_lifetimes.get(&def) else { continue };
        let spans = spans;
        for (k, n) in names.iter().enumerate() {
            if !used[i].lts[k] {
                errs.push(Diag::new(
                    "L0110",
                    spans.get(k).copied().unwrap_or(p.adts[id.0 as usize].span),
                    format!("lifetime parameter `'{}` is never used", p.interner.get(*n)),
                    "Use the lifetime in a field type, or remove it.",
                ));
            }
        }
    }
}

/// Every type param index that occurs in `t` (through any type form).
pub fn params_in(p: &Program, t: Ty) -> Vec<u32> {
    let slot = crate::intern::FxMap::default();
    let mut all = Mark { p, slot: &slot, used: &[], tys: Vec::new(), lts: Vec::new() };
    all.walk(t);
    all.tys
}

/// The field types of one Adt template, regioned where the borrow
/// tables have them (lifetime args first in user Adt args), else erased.
fn field_tys(p: &Program, id: AdtId) -> Vec<Ty> {
    let def = p.gx.adt_info[id.0 as usize].0;
    let regioned = p.regions.adt_fields.get(&def);
    let mut out = Vec::new();
    for (vi, v) in p.adts[id.0 as usize].variants.iter().enumerate() {
        for (fi, f) in v.fields.iter().enumerate() {
            out.push(regioned.and_then(|r| r.get(vi)).and_then(|r| r.get(fi)).copied().unwrap_or(f.ty));
        }
    }
    out
}

/// One walk of a field type: the params it uses under the current
/// `used` tables of the other Adts. With empty tables it lists every
/// param that occurs at all (the "only used recursively" test).
struct Mark<'x> {
    p: &'x Program,
    slot: &'x crate::intern::FxMap<AdtId, usize>,
    used: &'x [Used],
    tys: Vec<u32>,
    lts: Vec<u32>,
}

impl Mark<'_> {
    fn region(&mut self, r: Region) {
        if let Region::Param(i) = r {
            self.lts.push(i);
        }
    }

    fn arg(&mut self, g: GenericArg) {
        match g {
            GenericArg::Ty(t) => self.walk(t),
            GenericArg::Lifetime(r) => self.region(r),
            GenericArg::Const(_) => {}
        }
    }

    fn trait_ref(&mut self, tr: &TraitRef) {
        let tys = &self.p.tys;
        let args: Vec<GenericArg> = tys.args(tr.args).to_vec();
        let bound: Vec<GenericArg> = tys.bindings(tr.bindings).iter().map(|&(_, g)| g).collect();
        for g in args.into_iter().chain(bound) {
            self.arg(g);
        }
    }

    fn walk(&mut self, t: Ty) {
        let tys = &self.p.tys;
        match tys.kind(t) {
            TyKind::Param(ParamId(i)) => self.tys.push(i),
            TyKind::Ref(r, _, e) => {
                self.region(r);
                self.walk(e);
            }
            TyKind::RawPtr(_, e) | TyKind::Slice(e) | TyKind::Array(e, _) => self.walk(e),
            TyKind::Tuple(ts) => {
                for e in tys.tys_of(ts).to_vec() {
                    self.walk(e);
                }
            }
            TyKind::Adt(def, a) => {
                let args: Vec<GenericArg> = tys.args(a).to_vec();
                // A user Adt of this collect passes on only the args at
                // positions its own fields use (lifetimes first when the
                // type carries them, as in the regioned field types).
                if let Some(local) = tys.local_adt_of(def)
                    && let Some(&s) = self.slot.get(&local)
                    && s < self.used.len()
                {
                    let u = &self.used[s];
                    let n_lts = args.iter().filter(|g| matches!(g, GenericArg::Lifetime(_))).count();
                    for (k, g) in args.into_iter().enumerate() {
                        let keep =
                            if k < n_lts { u.lts.get(k).copied().unwrap_or(true) } else { u.tys.get(k - n_lts).copied().unwrap_or(true) };
                        if keep {
                            self.arg(g);
                        }
                    }
                } else {
                    for g in args {
                        self.arg(g);
                    }
                }
            }
            TyKind::FnPtr(sig) => {
                let b = tys.sig(sig).value;
                let ins: Vec<Ty> = tys.tys_of(b.inputs).to_vec();
                for e in ins {
                    self.walk(e);
                }
                self.walk(b.output);
            }
            TyKind::Dyn(b, r) => {
                let b = tys.dyn_bounds(b).clone();
                if let Some(pr) = &b.principal {
                    self.trait_ref(&pr.value);
                }
                for s in b.supertraits.iter() {
                    self.trait_ref(&s.value);
                }
                self.region(r);
            }
            TyKind::FnDef(_, a) | TyKind::Closure(_, a) | TyKind::Opaque(_, a) | TyKind::Projection(_, a) => {
                for g in tys.args(a).to_vec() {
                    self.arg(g);
                }
            }
            _ => {}
        }
    }
}
