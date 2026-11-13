//! Port of Effect-TS/tsgo `internal/rulerunner`: runs the enabled rules on
//! one source file and applies the configured severities and directives.

use crate::effect::diag;
use crate::effect::directives::{
    Directive, DirectiveSet, build_directive_set, to_category,
};
use crate::effect::etscore::{
    Severity, 
    is_command_line_mode,
};
use crate::effect::pluginoptions::{
    
};
use crate::effect::rule::{UNUSED_DIRECTIVE_NAME};
use crate::effect::rules;
use crate::gostd::Context;
use crate::prelude::*;


// Go: rulerunner.MinVisibleSeverity

// PORT: not in Effect-TS/tsgo, whose checkers run the rules in every mode,
// so its API answers carry Effect diagnostics. A standalone API process
// (`tsgo --api`) answers here as plain tsgo: no Effect rules and no Effect
// diagnostics (state note effectapi1-2026-10-07). An API session inside a
// language server shares the server's checkers, so it keeps the rules.
static API_RULES_OFF: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Marks this process as a standalone API process, where the Effect rules
/// and the plugin option check are off unless `TSGO_EFFECT_API=1` (read
/// once). `api::new_standalone_session` calls it.
pub fn set_api_process() {
    static EFFECT_API: std::sync::LazyLock<bool> =
        std::sync::LazyLock::new(|| std::env::var_os("TSGO_EFFECT_API").is_some_and(|v| v == "1"));
    API_RULES_OFF.store(!*EFFECT_API, std::sync::atomic::Ordering::Relaxed);
}

/// Test hook: undoes `set_api_process`, so this process runs the rules as
/// tsc does.
#[doc(hidden)]
pub fn clear_api_process() {
    API_RULES_OFF.store(false, std::sync::atomic::Ordering::Relaxed);
}


// Go: rulerunner.Run





