//! Port of Effect-TS/tsgo `internal/rules/global_timers.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: rules/global_timers.go globalTimerAlternative
#[derive(Clone, Copy, Debug)]
pub struct GlobalTimerAlternative {
    pub name: &'static str,
    pub alternative: &'static str,
}

// PORT: Go keeps a `map[string]globalTimerAlternative` and ranges over it in
// random order. The global symbols of `setTimeout` and `setInterval` differ,
// and the match loop breaks on the first match, so the order does not change
// the output.
// Go: rules/global_timers.go globalTimerAlternatives
static GLOBAL_TIMER_ALTERNATIVES: &[(&str, GlobalTimerAlternative)] = &[
    (
        "setTimeout",
        GlobalTimerAlternative {
            name: "setTimeout",
            alternative: "Effect.sleep or Schedule",
        },
    ),
    (
        "setInterval",
        GlobalTimerAlternative {
            name: "setInterval",
            alternative: "Schedule or Effect.repeat",
        },
    ),
];

// Go: rules/global_timers.go GlobalTimers


// Go: rules/global_timers.go GlobalTimersInEffect


// Go: rules/global_timers.go runGlobalTimers
