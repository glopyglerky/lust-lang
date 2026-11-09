//! Port of typescript-go `internal/parser/references.go`: the imports,
//! module augmentations and ambient module names of a parsed file.
//!
//! PORT: Go writes these to `*ast.SourceFile`. Here they go to the matching
//! `ParsedSourceFile` fields (plan contract 6).

use crate::frontend::prelude::*;

// Go: core/nodemodules.go:9 UnprefixedNodeCoreModules
// PORT: Go `map[string]bool`; a fixed array searched with `contains`.
// core/nodemodules.go is not in this unit, so the set is private here.
const UNPREFIXED_NODE_CORE_MODULES: [&str; 54] = [
    "assert",
    "assert/strict",
    "async_hooks",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "diagnostics_channel",
    "dns",
    "dns/promises",
    "domain",
    "events",
    "fs",
    "fs/promises",
    "http",
    "http2",
    "https",
    "inspector",
    "inspector/promises",
    "module",
    "net",
    "os",
    "path",
    "path/posix",
    "path/win32",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "readline/promises",
    "repl",
    "stream",
    "stream/consumers",
    "stream/promises",
    "stream/web",
    "string_decoder",
    "sys",
    "timers",
    "timers/promises",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "util/types",
    "v8",
    "vm",
    "wasi",
    "worker_threads",
    "zlib",
];

// Go: core/nodemodules.go:67 ExclusivelyPrefixedNodeCoreModules
// PORT: see UNPREFIXED_NODE_CORE_MODULES.
const EXCLUSIVELY_PREFIXED_NODE_CORE_MODULES: [&str; 5] = [
    "node:quic",
    "node:sea",
    "node:sqlite",
    "node:test",
    "node:test/reporters",
];

// Go: parser/references.go:11 collectExternalModuleReferences

// Go: parser/references.go:24 collectModuleReferences
