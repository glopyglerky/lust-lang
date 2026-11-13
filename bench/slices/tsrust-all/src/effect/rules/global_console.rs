//! Port of Effect-TS/tsgo `internal/rules/global_console.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// PORT: Go keeps a `map[string]string` and ranges over it in random order.
// A call expression matches at most one `console.<method>`, and the loop
// breaks on the first match, so the order does not change the output.
// Go: rules/global_console.go globalConsoleMethodAlternatives
static GLOBAL_CONSOLE_METHOD_ALTERNATIVES: &[(&str, &str)] = &[
    ("log", "Effect.log or Logger"),
    ("warn", "Effect.logWarning or Logger"),
    ("error", "Effect.logError or Logger"),
    ("info", "Effect.logInfo or Logger"),
    ("debug", "Effect.logDebug or Logger"),
    ("trace", "Effect.logTrace or Logger"),
];

// Go: rules/global_console.go GlobalConsole


// Go: rules/global_console.go GlobalConsoleInEffect


// Go: rules/global_console.go runGlobalConsole
