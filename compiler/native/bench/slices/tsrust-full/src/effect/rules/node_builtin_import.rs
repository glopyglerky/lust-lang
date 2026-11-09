//! Port of Effect-TS/tsgo `internal/rules/node_builtin_import.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

#[derive(Clone, Copy, Debug)]
struct ModuleAlternative {
    alternative: &'static str,
    package: &'static str,
    module: &'static str,
}

// PORT: Go maps `map[string]moduleAlternative`; the port keeps the entries in
// slices and looks them up by key (`lookup_module_alternative`).
static MODULE_ALTERNATIVES_V3: &[(&str, ModuleAlternative)] = &[
    (
        "fs",
        ModuleAlternative {
            alternative: "FileSystem",
            package: "@effect/platform",
            module: "fs",
        },
    ),
    (
        "node:fs",
        ModuleAlternative {
            alternative: "FileSystem",
            package: "@effect/platform",
            module: "fs",
        },
    ),
    (
        "fs/promises",
        ModuleAlternative {
            alternative: "FileSystem",
            package: "@effect/platform",
            module: "fs",
        },
    ),
    (
        "node:fs/promises",
        ModuleAlternative {
            alternative: "FileSystem",
            package: "@effect/platform",
            module: "fs",
        },
    ),
    (
        "path",
        ModuleAlternative {
            alternative: "Path",
            package: "@effect/platform",
            module: "path",
        },
    ),
    (
        "node:path",
        ModuleAlternative {
            alternative: "Path",
            package: "@effect/platform",
            module: "path",
        },
    ),
    (
        "path/posix",
        ModuleAlternative {
            alternative: "Path",
            package: "@effect/platform",
            module: "path",
        },
    ),
    (
        "node:path/posix",
        ModuleAlternative {
            alternative: "Path",
            package: "@effect/platform",
            module: "path",
        },
    ),
    (
        "path/win32",
        ModuleAlternative {
            alternative: "Path",
            package: "@effect/platform",
            module: "path",
        },
    ),
    (
        "node:path/win32",
        ModuleAlternative {
            alternative: "Path",
            package: "@effect/platform",
            module: "path",
        },
    ),
    (
        "child_process",
        ModuleAlternative {
            alternative: "CommandExecutor",
            package: "@effect/platform",
            module: "child_process",
        },
    ),
    (
        "node:child_process",
        ModuleAlternative {
            alternative: "CommandExecutor",
            package: "@effect/platform",
            module: "child_process",
        },
    ),
    (
        "http",
        ModuleAlternative {
            alternative: "HttpClient",
            package: "@effect/platform",
            module: "http",
        },
    ),
    (
        "node:http",
        ModuleAlternative {
            alternative: "HttpClient",
            package: "@effect/platform",
            module: "http",
        },
    ),
    (
        "https",
        ModuleAlternative {
            alternative: "HttpClient",
            package: "@effect/platform",
            module: "https",
        },
    ),
    (
        "node:https",
        ModuleAlternative {
            alternative: "HttpClient",
            package: "@effect/platform",
            module: "https",
        },
    ),
    (
        "console",
        ModuleAlternative {
            alternative: "Console",
            package: "effect",
            module: "console",
        },
    ),
    (
        "node:console",
        ModuleAlternative {
            alternative: "Console",
            package: "effect",
            module: "console",
        },
    ),
    (
        "timers",
        ModuleAlternative {
            alternative: "Effect",
            package: "effect",
            module: "timers",
        },
    ),
    (
        "node:timers",
        ModuleAlternative {
            alternative: "Effect",
            package: "effect",
            module: "timers",
        },
    ),
    (
        "timers/promises",
        ModuleAlternative {
            alternative: "Effect",
            package: "effect",
            module: "timers",
        },
    ),
    (
        "node:timers/promises",
        ModuleAlternative {
            alternative: "Effect",
            package: "effect",
            module: "timers",
        },
    ),
    (
        "stream",
        ModuleAlternative {
            alternative: "Stream",
            package: "effect",
            module: "stream",
        },
    ),
    (
        "node:stream",
        ModuleAlternative {
            alternative: "Stream",
            package: "effect",
            module: "stream",
        },
    ),
    (
        "stream/promises",
        ModuleAlternative {
            alternative: "Stream",
            package: "effect",
            module: "stream",
        },
    ),
    (
        "node:stream/promises",
        ModuleAlternative {
            alternative: "Stream",
            package: "effect",
            module: "stream",
        },
    ),
    (
        "stream/web",
        ModuleAlternative {
            alternative: "Stream",
            package: "effect",
            module: "stream",
        },
    ),
    (
        "node:stream/web",
        ModuleAlternative {
            alternative: "Stream",
            package: "effect",
            module: "stream",
        },
    ),
];

static MODULE_ALTERNATIVES_V4: &[(&str, ModuleAlternative)] = &[
    (
        "fs",
        ModuleAlternative {
            alternative: "FileSystem",
            package: "effect",
            module: "fs",
        },
    ),
    (
        "node:fs",
        ModuleAlternative {
            alternative: "FileSystem",
            package: "effect",
            module: "fs",
        },
    ),
    (
        "fs/promises",
        ModuleAlternative {
            alternative: "FileSystem",
            package: "effect",
            module: "fs",
        },
    ),
    (
        "node:fs/promises",
        ModuleAlternative {
            alternative: "FileSystem",
            package: "effect",
            module: "fs",
        },
    ),
    (
        "path",
        ModuleAlternative {
            alternative: "Path",
            package: "effect",
            module: "path",
        },
    ),
    (
        "node:path",
        ModuleAlternative {
            alternative: "Path",
            package: "effect",
            module: "path",
        },
    ),
    (
        "path/posix",
        ModuleAlternative {
            alternative: "Path",
            package: "effect",
            module: "path",
        },
    ),
    (
        "node:path/posix",
        ModuleAlternative {
            alternative: "Path",
            package: "effect",
            module: "path",
        },
    ),
    (
        "path/win32",
        ModuleAlternative {
            alternative: "Path",
            package: "effect",
            module: "path",
        },
    ),
    (
        "node:path/win32",
        ModuleAlternative {
            alternative: "Path",
            package: "effect",
            module: "path",
        },
    ),
    (
        "child_process",
        ModuleAlternative {
            alternative: "ChildProcess",
            package: "effect/unstable/process",
            module: "child_process",
        },
    ),
    (
        "node:child_process",
        ModuleAlternative {
            alternative: "ChildProcess",
            package: "effect/unstable/process",
            module: "child_process",
        },
    ),
    (
        "http",
        ModuleAlternative {
            alternative: "HttpClient",
            package: "effect/unstable/http",
            module: "http",
        },
    ),
    (
        "node:http",
        ModuleAlternative {
            alternative: "HttpClient",
            package: "effect/unstable/http",
            module: "http",
        },
    ),
    (
        "https",
        ModuleAlternative {
            alternative: "HttpClient",
            package: "effect/unstable/http",
            module: "https",
        },
    ),
    (
        "node:https",
        ModuleAlternative {
            alternative: "HttpClient",
            package: "effect/unstable/http",
            module: "https",
        },
    ),
    (
        "console",
        ModuleAlternative {
            alternative: "Console",
            package: "effect",
            module: "console",
        },
    ),
    (
        "node:console",
        ModuleAlternative {
            alternative: "Console",
            package: "effect",
            module: "console",
        },
    ),
    (
        "timers",
        ModuleAlternative {
            alternative: "Effect",
            package: "effect",
            module: "timers",
        },
    ),
    (
        "node:timers",
        ModuleAlternative {
            alternative: "Effect",
            package: "effect",
            module: "timers",
        },
    ),
    (
        "timers/promises",
        ModuleAlternative {
            alternative: "Effect",
            package: "effect",
            module: "timers",
        },
    ),
    (
        "node:timers/promises",
        ModuleAlternative {
            alternative: "Effect",
            package: "effect",
            module: "timers",
        },
    ),
    (
        "stream",
        ModuleAlternative {
            alternative: "Stream",
            package: "effect",
            module: "stream",
        },
    ),
    (
        "node:stream",
        ModuleAlternative {
            alternative: "Stream",
            package: "effect",
            module: "stream",
        },
    ),
    (
        "stream/promises",
        ModuleAlternative {
            alternative: "Stream",
            package: "effect",
            module: "stream",
        },
    ),
    (
        "node:stream/promises",
        ModuleAlternative {
            alternative: "Stream",
            package: "effect",
            module: "stream",
        },
    ),
    (
        "stream/web",
        ModuleAlternative {
            alternative: "Stream",
            package: "effect",
            module: "stream",
        },
    ),
    (
        "node:stream/web",
        ModuleAlternative {
            alternative: "Stream",
            package: "effect",
            module: "stream",
        },
    ),
    (
        "crypto",
        ModuleAlternative {
            alternative: "Crypto",
            package: "effect",
            module: "crypto",
        },
    ),
    (
        "node:crypto",
        ModuleAlternative {
            alternative: "Crypto",
            package: "effect",
            module: "crypto",
        },
    ),
];
fn lookup_module_alternative(
    alternatives: &[(&str, ModuleAlternative)],
    specifier: &str,
) -> Option<ModuleAlternative> {
    alternatives
        .iter()
        .find(|(key, _)| *key == specifier)
        .map(|(_, alt)| *alt)
}



#[derive(Clone, Debug)]
pub struct NodeBuiltinImportMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub alternative: String,
    pub package: String,
    pub module: String,
}

