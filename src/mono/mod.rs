// Monomorphization instance discovery (docs/ARCH.md section 4.1
// and 4.2, basic form; W-TRAITS-CORE in wave A, W-MONO later).
//
// The walk starts at the roots (`main`, test fns, exported fns,
// vtable sets, drop glue) and discovers every reachable instance.
// An instance is keyed by its template def plus concrete type and
// const args, regions erased (`driver::graph::InstanceKey`). Each
// instance caches its resolved edge list (the callee instance keys
// its body references) so an edit rewalks only from dirty
// instances; `walk` takes that cache in and returns it updated.
//
// This module is the discovery half only. Turning one instance
// into check requests and concrete code is the caller's `discover`
// step, which collect and the checker provide; codegen of the
// resulting instances is W-CGEN's, through the usual per-fn path.
//
// iface: W-DRIVER assigns slots from `MonoGraph::order` (discovery
// order, roots first) keyed by `InstanceKeyHash`; W-CGEN receives
// one concrete body per instance exactly as it receives one per
// non-generic fn today.
pub mod vtable;

use crate::def::DefId;
use crate::driver::graph::InstanceKey;
use crate::intern::FxMap;
use crate::traits::ImplId;

/// The recursion depth cap of instantiation (section 4.2), matching
/// rustc's default. Exceeding it is a build error, not a check
/// error: `lust check` does not walk instances.
pub const MAX_INSTANCE_DEPTH: usize = 128;

/// The default per-crate instance cap. A stop against hangs, not a
/// language limit; the error names the worst template.
pub const DEFAULT_MAX_INSTANCES: usize = 2_000_000;

/// Why a walk stopped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MonoError {
    /// Instantiation recursed past the depth cap. The payload is
    /// the instantiation chain, outermost first, as rustc prints it.
    RecursionLimit(Vec<InstanceKey>),
    /// The instance cap was reached. The payload is the template
    /// with the most instances and its count.
    TooManyInstances { worst: DefId, count: usize },
}

/// What one discovered instance records for the cache: its resolved
/// callee edges, and the impls selection picked on those edges.
/// Selected impls are part of the instance's code identity (its
/// fingerprint input, section 4.2), never of its key.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstanceEdges {
    pub callees: Vec<InstanceKey>,
    pub selected_impls: Vec<ImplId>,
}

/// The result of a walk: every reachable instance in discovery
/// order (roots first, each instance after the one that first
/// reached it), plus the updated edge cache.
#[derive(Clone, Debug, Default)]
pub struct MonoGraph {
    pub order: Vec<InstanceKey>,
    pub edges: FxMap<InstanceKey, InstanceEdges>,
}

impl MonoGraph {
    /// The number of instances of one template in this graph.
    pub fn instances_of(&self, def: DefId) -> usize {
        self.order.iter().filter(|k| k.def == def).count()
    }

    /// Per-template instance counts, most instances first. The
    /// report line W-TRAITS-CORE and W-MONO quote per crate.
    pub fn fanout(&self) -> Vec<(DefId, usize)> {
        let mut counts: FxMap<DefId, usize> = FxMap::default();
        for k in &self.order {
            *counts.entry(k.def).or_default() += 1;
        }
        let mut out: Vec<(DefId, usize)> = counts.into_iter().collect();
        out.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.hash.0.cmp(&b.0.hash.0)));
        out
    }
}

/// Walks instances from `roots`. `discover` returns the edges of
/// one instance (its callees and selected impls); it runs only for
/// instances missing from `cache`, or listed in `dirty`, and its
/// result replaces the cache entry. A cached instance that an edit
/// left clean costs one map probe, so the walk is incremental from
/// the first version, as section 4.2 requires.
///
/// Recursion is judged on the discovery stack: an instance that
/// reaches itself through ever-changing args grows the stack until
/// the depth cap stops it, exactly like rustc's post-mono error.
pub fn walk(
    roots: &[InstanceKey],
    cache: FxMap<InstanceKey, InstanceEdges>,
    dirty: &[InstanceKey],
    max_instances: usize,
    mut discover: impl FnMut(&InstanceKey) -> InstanceEdges,
) -> Result<MonoGraph, MonoError> {
    let mut graph = MonoGraph { order: Vec::new(), edges: cache };
    let mut seen: FxMap<InstanceKey, ()> = FxMap::default();
    // Iterative DFS with an explicit stack of (key, next edge
    // index), so the recursion chain is available for the error
    // and deep instance graphs do not grow the call stack.
    for root in roots {
        let mut stack: Vec<(InstanceKey, usize)> = Vec::new();
        push_new(&mut graph, &mut seen, &mut stack, *root, dirty, &mut discover)?;
        while let Some(&(key, next)) = stack.last() {
            let callees_len = graph.edges.get(&key).map_or(0, |e| e.callees.len());
            if next >= callees_len {
                stack.pop();
                continue;
            }
            stack.last_mut().unwrap().1 += 1;
            let callee = graph.edges[&key].callees[next];
            if seen.contains_key(&callee) {
                // Already discovered. A back edge onto the stack is
                // ordinary recursion with fixed args, which is
                // legal; only unbounded growth errors (below).
                continue;
            }
            if stack.len() >= MAX_INSTANCE_DEPTH {
                let mut chain: Vec<InstanceKey> = stack.iter().map(|(k, _)| *k).collect();
                chain.push(callee);
                return Err(MonoError::RecursionLimit(chain));
            }
            if graph.order.len() >= max_instances {
                let (worst, count) = graph.fanout().into_iter().next().unwrap_or((callee.def, 0));
                return Err(MonoError::TooManyInstances { worst, count });
            }
            push_new(&mut graph, &mut seen, &mut stack, callee, dirty, &mut discover)?;
        }
    }
    // The cache keeps only reachable instances: an edit that makes
    // an instance unreachable also frees its edge entry here, so
    // the cache cannot grow without bound across edits. (Its slot
    // stays dead until the next base rebuild; that is W-DRIVER's
    // append-only slot policy, not this walk's.)
    graph.edges.retain(|k, _| seen.contains_key(k));
    Ok(graph)
}

/// Records one newly reached instance: refreshes its edges when it
/// is missing or dirty, and pushes it on the walk stack.
fn push_new(
    graph: &mut MonoGraph,
    seen: &mut FxMap<InstanceKey, ()>,
    stack: &mut Vec<(InstanceKey, usize)>,
    key: InstanceKey,
    dirty: &[InstanceKey],
    discover: &mut impl FnMut(&InstanceKey) -> InstanceEdges,
) -> Result<(), MonoError> {
    if seen.contains_key(&key) {
        return Ok(());
    }
    seen.insert(key, ());
    graph.order.push(key);
    if !graph.edges.contains_key(&key) || dirty.contains(&key) {
        let edges = discover(&key);
        graph.edges.insert(key, edges);
    }
    stack.push((key, 0));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::def::{DefPathHash, LOCAL_CRATE};
    use crate::ty::{ArgsId, EMPTY_ARGS};

    fn key(def_hash: u64, arg: u32) -> InstanceKey {
        InstanceKey { def: DefId { krate: LOCAL_CRATE, hash: DefPathHash(def_hash) }, args: ArgsId(arg) }
    }

    fn edges(callees: Vec<InstanceKey>) -> InstanceEdges {
        InstanceEdges { callees, selected_impls: Vec::new() }
    }

    /// Discovery visits each instance once, in DFS order from the
    /// roots, and a self-recursive instance with fixed args is a
    /// legal back edge, not an overflow.
    #[test]
    fn walk_discovers_each_instance_once() {
        let (a, b, c) = (key(1, 0), key(2, 0), key(2, 1));
        let discover = |k: &InstanceKey| {
            if *k == a {
                edges(vec![b, c])
            } else if *k == b {
                edges(vec![b])
            } else {
                edges(vec![])
            }
        };
        let g = walk(&[a], FxMap::default(), &[], 100, discover).unwrap();
        assert_eq!(g.order, [a, b, c]);
        assert_eq!(g.instances_of(b.def), 2);
        assert_eq!(g.fanout()[0], (b.def, 2));
    }

    /// Cached edges are reused without discovery; a dirty instance
    /// is rediscovered and unreachable cache entries are dropped.
    #[test]
    fn walk_reuses_clean_cache() {
        let (a, b, dead) = (key(1, 0), key(2, 0), key(9, 0));
        let mut cache = FxMap::default();
        cache.insert(a, edges(vec![b]));
        cache.insert(b, edges(vec![]));
        cache.insert(dead, edges(vec![]));
        let mut discovered: Vec<InstanceKey> = Vec::new();
        let g = walk(&[a], cache, &[b], 100, |k| {
            discovered.push(*k);
            edges(vec![])
        })
        .unwrap();
        assert_eq!(discovered, [b], "only the dirty instance is rediscovered");
        assert!(!g.edges.contains_key(&dead), "unreachable cache entries are dropped");
    }

    /// Unbounded instantiation (`f<T>` reaching `f<Vec<T>>` style,
    /// modelled as ever-new keys) stops at the depth cap with the
    /// chain intact.
    #[test]
    fn walk_stops_at_recursion_limit() {
        let mut next = 0u32;
        let err = walk(&[key(1, 0)], FxMap::default(), &[], usize::MAX, |_| {
            next += 1;
            edges(vec![key(1, next)])
        })
        .unwrap_err();
        match err {
            MonoError::RecursionLimit(chain) => assert_eq!(chain.len(), MAX_INSTANCE_DEPTH + 1),
            other => panic!("expected a recursion limit, got {other:?}"),
        }
        let _ = EMPTY_ARGS;
    }
}
