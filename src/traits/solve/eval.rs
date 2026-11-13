// rustc's evaluation of one obligation through every candidate (the old
// solver's `evaluate_candidate`), for one question only: does it pass the
// recursion limit (E0275)? Selection (the parent module) never searches
// and picks the param env without looking at impls; rustc instead
// evaluates each matching impl's own predicates, recursively, whenever
// more than one candidate applies. That is how an impl whose where clause
// asks the same trait at a growing type (`impl<T> Tr for G<T> where
// G<Option<T>>: Tr`) overflows at its well-formedness check, used or not.
use super::*;

/// The answer of one evaluation. Only `Overflow` is acted on; the other
/// three steer the walk as rustc's results do (a failed nested predicate
/// stops its candidate, so a later sibling is never evaluated).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Eval {
    Holds,
    Fails,
    /// A cycle, an inference variable as Self, or the step budget: may
    /// hold, never an overflow.
    Unknown,
    Overflow,
}

/// One evaluation walk. `limit` is the crate's recursion limit; an
/// obligation `level` steps below the root has rustc's recursion depth
/// `level + 1` (the root's stack entry counts once), and passing the
/// limit is the overflow.
pub struct Evaluator {
    limit: usize,
    stack: Vec<TraitRef>,
    memo: FxMap<TraitRef, Eval>,
    /// Probes left before the walk gives up as `Unknown` (rustc can take
    /// exponential time on branching growth; lust must never hang).
    budget: usize,
}

/// Probes one walk may run.
const BUDGET: usize = 20_000;

impl Evaluator {
    pub fn new(limit: usize) -> Evaluator {
        Evaluator { limit, stack: Vec::new(), memo: FxMap::default(), budget: BUDGET }
    }

    /// Evaluates a root obligation the param env already proves (a where
    /// clause of the item being checked) through its impl candidates, as
    /// rustc's winnowing does when the env and an impl both match. True
    /// when that walk passes the recursion limit.
    pub fn where_clause_overflows<C: SolveCx>(
        &mut self,
        solver: &mut Solver,
        cx: &mut C,
        index: &ImplIndex,
        env: &[Predicate],
        tr: TraitRef,
    ) -> bool {
        self.eval(solver, cx, index, env, tr, 0) == Eval::Overflow
    }

    fn eval<C: SolveCx>(
        &mut self,
        solver: &mut Solver,
        cx: &mut C,
        index: &ImplIndex,
        env: &[Predicate],
        tr: TraitRef,
        level: usize,
    ) -> Eval {
        if level > 0 && level + 1 > self.limit {
            return Eval::Overflow;
        }
        let tr = deep_trait_ref(cx, tr);
        if let Some(&e) = self.memo.get(&tr) {
            return e;
        }
        if self.stack.contains(&tr) || self.budget == 0 {
            return Eval::Unknown;
        }
        self.budget -= 1;
        self.stack.push(tr);
        let (out, exact) = self.eval_candidates(solver, cx, index, env, tr, level);
        self.stack.pop();
        // An answer that read a cycle on the stack is provisional.
        let tys = cx.tys();
        if exact && !tys.args(tr.args).iter().any(|g| g.as_ty().is_some_and(|t| tys.flags(t).contains_infer())) {
            self.memo.insert(tr, out);
        }
        out
    }

    /// Every candidate of one obligation: the env, builtins, and each impl
    /// whose header matches, with that impl's predicates evaluated one
    /// level down in written order. The bool is false when a part of the
    /// answer was `Unknown`.
    fn eval_candidates<C: SolveCx>(
        &mut self,
        solver: &mut Solver,
        cx: &mut C,
        index: &ImplIndex,
        env: &[Predicate],
        tr: TraitRef,
        level: usize,
    ) -> (Eval, bool) {
        let self_ty = self_ty_of(cx.tys(), &tr);
        let self_deep = cx.deep_resolve(self_ty);
        if matches!(cx.tys().kind(self_deep), TyKind::Infer(_)) {
            return (Eval::Unknown, false);
        }
        let mut holds = false;
        for pred in env {
            let Predicate::Trait(b) = pred else { continue };
            if b.count != 0 || b.value.def != tr.def {
                continue;
            }
            let snap = cx.snapshot();
            holds |= unify_trait_refs(cx, b.value, tr);
            cx.rollback(snap);
        }
        // Builtin answers end the walk here: closures, `Sized`, the auto
        // traits, and the derive family on structural types count as
        // holding. Only a failure stops a candidate's later predicates,
        // so reading these as holds never stops a walk rustc continues.
        let snap = cx.snapshot();
        let builtin = cx.closure_select(tr).or_else(|| cx.builtin_select(tr));
        cx.rollback(snap);
        if let Some(sel) = builtin {
            return (if matches!(sel, Selection::NoImpl) && !holds { Eval::Fails } else { Eval::Holds }, true);
        }
        let structural_self = matches!(
            cx.tys().kind(self_deep),
            TyKind::Tuple(_) | TyKind::Array(..) | TyKind::Slice(_) | TyKind::Ref(..) | TyKind::RawPtr(..) | TyKind::Closure(..)
        );
        if index.structural.contains(&tr.def) || index.derive_family.contains(&tr.def) && structural_self {
            return (Eval::Holds, true);
        }
        if let Some(h) = cx.structural(tr) {
            return (if h || holds { Eval::Holds } else { Eval::Fails }, true);
        }
        let mut exact = true;
        let mut unknown = false;
        for id in index.candidates(cx.tys(), tr.def, self_deep, &|t| t) {
            let rec = index.get(id);
            let snap = cx.snapshot();
            let Some((inst, _)) = solver.probe_impl(cx, index, env, rec, tr) else {
                cx.rollback(snap);
                continue;
            };
            let preds: Vec<Predicate> = rec.generics.predicates.iter().map(|&p| subst_predicate(cx.tys(), p, &inst)).collect();
            let mut cand = Eval::Holds;
            for p in preds {
                let Predicate::Trait(b) = p else { continue };
                if b.count != 0 {
                    continue;
                }
                match self.eval(solver, cx, index, env, b.value, level + 1) {
                    Eval::Overflow => {
                        cx.rollback(snap);
                        return (Eval::Overflow, true);
                    }
                    Eval::Fails => {
                        cand = Eval::Fails;
                        break;
                    }
                    Eval::Unknown => {
                        cand = Eval::Unknown;
                        exact = false;
                    }
                    Eval::Holds => {}
                }
            }
            cx.rollback(snap);
            holds |= cand == Eval::Holds;
            unknown |= cand == Eval::Unknown;
        }
        let out = if holds {
            Eval::Holds
        } else if unknown {
            Eval::Unknown
        } else {
            Eval::Fails
        };
        (out, exact)
    }
}
