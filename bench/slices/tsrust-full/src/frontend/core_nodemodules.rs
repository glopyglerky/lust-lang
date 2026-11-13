//! Port of Go `core/nodemodules.go`.

use crate::frontend::prelude::*;
use std::sync::LazyLock;

// Go: core/nodemodules.go:9 UnprefixedNodeCoreModules
// require('module').builtinModules.filter(x => !x.match(/^(?:_|node:)/))
// PORT: Go package var `map[string]bool`. `IndexMap` keeps the Go list order.
// A missing key reads as `false` in Go: use `.get(k).copied().unwrap_or(false)`
// or `contains_key` (every value is `true`).

// Go: core/nodemodules.go:67 ExclusivelyPrefixedNodeCoreModules
// require('module').builtinModules.filter(x => x.startsWith('node:'))
// PORT: see `UNPREFIXED_NODE_CORE_MODULES`.

// Go: core/nodemodules.go:75 NodeCoreModules
// PORT: Go `sync.OnceValue(func() map[string]bool)` is a function over a
// `LazyLock`. Go ranges over a map (random order); this inserts in the Go
// list order. The result is only used for lookups.

// Go: core/nodemodules.go:85 NonRelativeModuleNameForTypingCache
