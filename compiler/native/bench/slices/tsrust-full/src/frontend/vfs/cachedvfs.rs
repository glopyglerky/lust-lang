//! Go: internal/vfs/cachedvfs/cachedvfs.go

use crate::frontend::prelude::*;
use std::cell::Cell;
use std::time::SystemTime;

// Go: cachedvfs.go:11 FS
// PORT: the Go type `cachedvfs.FS` is `CachedFs`. `atomic.Bool` and
// `collections.SyncMap` become `Cell` and `RefCell<FxHashMap>` (the port is
// single-threaded). The stat cache stores Go nil results as `None`.

// Go: cachedvfs.go:24 From
// PORT: the Go package function `cachedvfs.From` is `cachedvfs_from`.

// PORT: the Go `SyncMap.Load`. The value is cloned so no borrow is held
// while the wrapped file system runs.
fn cache_load<V: Clone>(cache: &RefCell<FxHashMap<String, V>>, key: &str) -> Option<V> {
    cache.borrow().get(key).cloned()
}


// PORT: not in Go. The cached answer for `path`, with no call: `None` when
// the cache has none or is off. A language server load compares the
// answers of its resolve-ahead workers with it (project/compilerhost.rs).

