//! Go: internal/fswatch/pathcompare.go (ts#64210).
//!
//! PORT: paths are Go strings in the port form (see `gostring`): a byte
//! that is not valid UTF-8 is a marker unit of non-ASCII chars. The byte
//! loops see such a unit as non-ASCII, as Go sees the byte, and equal port
//! forms are equal Go strings. Go `*comparisonCache` (a pointer to a map
//! that one synchronous pass shares) is `Option<&Mutex<ComparisonCache>>`:
//! prepared paths are stored in callbacks, which cross threads, so the
//! borrow must be `Send`. Go allocates the nil map on first use; the port's
//! mutex holds an empty map.

use crate::fswatch::prelude::*;

use std::sync::Mutex;

// Go: pathcompare.go:8 pathComparer
// PORT: Go also has the exported `PathComparer` (pathkey.go); it is
// `PathComparerExported` (PORTING "Names").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PathComparer {
    pub ignore_case: bool,
}

// Go: pathcompare.go:14 comparisonPath
/// Watch roots are prepared before publication and are immutable thereafter.
/// Event paths are local to one routing operation and folded only on demand.
#[derive(Clone, Debug, Default)]
pub struct ComparisonPath<'c> {
    pub path: String,
    pub folded: String,
    pub ready: bool,
    pub cache: Option<&'c Mutex<ComparisonCache>>,
}

// Go: pathcompare.go:23 comparisonCache
/// Shared only within a synchronous callback/termination pass, never published
/// to a subscriber or stored on a watch.
pub type ComparisonCache = FxHashMap<String, String>;

impl PathComparer {
    // Go: pathcompare.go:25 pathComparer.prepare
    pub fn prepare(&self, path: &str) -> ComparisonPath<'static> {
        let mut p = ComparisonPath {
            path: path.to_string(),
            ..Default::default()
        };
        if self.ignore_case && NATIVE_PATH_FOLDING {
            p.fold();
        }
        p
    }
}

impl ComparisonPath<'_> {
    // Go: pathcompare.go:33 comparisonPath.fold
    pub fn fold(&mut self) -> String {
        if !self.ready {
            if let Some(cache) = self.cache {
                if let Some(folded) = cache.lock().unwrap().get(&self.path) {
                    self.folded = folded.clone();
                    self.ready = true;
                    return self.folded.clone();
                }
            }
            self.folded = fold_native_path(&self.path);
            self.ready = true;
            if let Some(cache) = self.cache {
                cache
                    .lock()
                    .unwrap()
                    .insert(self.path.clone(), self.folded.clone());
            }
        }
        self.folded.clone()
    }
}

impl PathComparer {
    // Go: pathcompare.go:54 pathComparer.suffix

    // Go: pathcompare.go:59 pathComparer.suffixPrepared
    // PORT: Go passes `root` by value; the port borrows it (suffixUnicode
    // folds a copy, as Go does).

    // Go: pathcompare.go:73 pathComparer.suffixUnicode
}

// Go: pathcompare.go:112 pathSuffixASCII

// Go: pathcompare.go:141 pathSuffixFoldUnicode

impl PathComparer {
    // Go: pathcompare.go:161 pathComparer.contains

    // Go: pathcompare.go:166 pathComparer.rebase

    // Go: pathcompare.go:171 pathComparer.rebasePrepared
    // PORT: Go passes `from` by value; the port borrows it.
}


