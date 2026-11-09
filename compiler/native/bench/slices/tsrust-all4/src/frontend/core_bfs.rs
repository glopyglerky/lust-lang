//! Go `internal/core/bfs.go`.
//!
//! PORT: the language service runs on one dispatch thread
//! (`project/dirty/interfaces.rs`, decision 1). Go runs each level's jobs in
//! goroutines; the port runs them serially in queue order. Go starts every
//! goroutine of a level before a slow `visit` returns, so in Go's common
//! schedule each job passes its start check (`core/bfs.go:102`) and is
//! visited. The port follows that schedule: it visits every job of the
//! level and keeps only the check after the visit (`core/bfs.go:128`), so
//! the result and the next level are Go's deterministic ones. The side
//! effects of `visit` (the project search creates and loads each project of
//! the level) are those of Go's common schedule.
//!
//! Go's other schedule has no condition that the port can read: a later
//! job skips its visit only when its goroutine runs the start check after
//! an earlier job's `visit` has stored `lowestGoal`. That can happen only
//! when the earlier visit is fast (a project that is already loaded and up
//! to date, as at a hono reopen), and then it depends on the scheduler and
//! the CPU: Go N skipped hono's spec project in 5 of 7 reopens on
//! mini-743d, took the common schedule in both runs on alvin
//! (projsearch1b), and skipped it in about 4 of 15 reopens for the R170
//! reviewer. Both answers are Go's. The port keeps the
//! common one, so its answers (willRenameFiles: 11 files, 2 tests) do not
//! change from run to run or host to host.

use crate::frontend::prelude::*;
use std::hash::Hash;

// Go: core/bfs.go:11 BreadthFirstSearchResult
pub struct BreadthFirstSearchResult<N> {
    pub stopped: bool,
    pub path: Vec<N>,
}

// Go: core/bfs.go:16 breadthFirstSearchJob
pub struct BreadthFirstSearchJob<N> {
    pub node: N,
    pub parent: Option<Rc<BreadthFirstSearchJob<N>>>,
}

// Go: core/bfs.go:21 BreadthFirstSearchLevel
// PORT: Go holds a pointer to the level's `OrderedMap`. The port moves the
// map in for the `PreprocessLevel` call and back out after it. `RefCell`
// lets a `Range` callback call `Delete` on the same level, as Go does.
pub struct BreadthFirstSearchLevel<K, N> {
    pub jobs: RefCell<IndexMap<K, Rc<BreadthFirstSearchJob<N>>>>,
}

impl<K: Eq + Hash, N: Clone> BreadthFirstSearchLevel<K, N> {
    // Go: core/bfs.go:25 Has

    // Go: core/bfs.go:29 Delete
    pub fn delete(&self, key: &K) {
        // PORT: Go OrderedMap.Delete keeps the order of the other keys.
        self.jobs.borrow_mut().shift_remove(key);
    }

    // Go: core/bfs.go:33 Range
}

// Go: core/bfs.go:41 BreadthFirstSearchOptions
pub struct BreadthFirstSearchOptions<'a, K, N> {
    // Visited is a set of nodes that have already been visited.
    // If nil, a new set will be created.
    // PORT: Go `*collections.SyncSet[K]`.
    pub visited: Option<&'a RefCell<FxHashSet<K>>>,
    // PreprocessLevel is a function that, if provided, will be called
    // before each level, giving the caller an opportunity to remove nodes.
    pub preprocess_level: Option<&'a mut dyn FnMut(&BreadthFirstSearchLevel<K, N>)>,
}

impl<K, N> Default for BreadthFirstSearchOptions<'_, K, N> {
    fn default() -> Self {
        BreadthFirstSearchOptions {
            visited: None,
            preprocess_level: None,
        }
    }
}

// Go: core/bfs.go:53 BreadthFirstSearchParallel
// BreadthFirstSearchParallel performs a breadth-first search on a graph
// starting from the given node. It processes nodes in parallel and returns the path
// from the first node that satisfies the `visit` function back to the start node.

// Go: core/bfs.go:76 result (local type of BreadthFirstSearchParallelEx)
struct ProcessLevelResult<K, N> {
    stop: bool,
    job: Option<Rc<BreadthFirstSearchJob<N>>>,
    next: Option<IndexMap<K, Rc<BreadthFirstSearchJob<N>>>>,
}

// Go: core/bfs.go:64 BreadthFirstSearchParallelEx
// BreadthFirstSearchParallelEx is an extension of BreadthFirstSearchParallel that allows
// the caller to pass a pre-seeded set of already-visited nodes and a preprocessing function
// that can be used to remove nodes from each level before parallel processing.

// Go: core/bfs.go:86 processLevel (closure in BreadthFirstSearchParallelEx)
// processLevel processes each node at the current level in parallel.
// It produces either a list of jobs to be processed in the next level,
// or a result if the visit function returns true for any node.
// PORT: a function with the closure's captures as parameters. The job
// goroutines run serially in queue order.

// Go: core/bfs.go:169 createPath (closure in BreadthFirstSearchParallelEx)
fn create_path<N: Clone>(job: Option<Rc<BreadthFirstSearchJob<N>>>) -> Vec<N> {
    let mut path = Vec::new();
    let mut job = job;
    while let Some(j) = job {
        path.push(j.node.clone());
        job = j.parent.clone();
    }
    path
}

// Go: core/bfs.go:196 updateMin
// updateMin updates the atomic integer `a` to the candidate value if it is less than the current value.
// PORT: one thread, so the compare-and-swap loop is a plain compare.
pub fn update_min(a: &mut i64, candidate: i64) -> bool {
    let current = *a;
    if current < candidate {
        return false;
    }
    *a = candidate;
    true
}
