//! Port of Effect-TS/tsgo `internal/rules/race_first_with_sleep_to_timeout.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



// Go: rules/race_first_with_sleep_to_timeout.go RaceFirstWithSleepToTimeoutMatch
#[derive(Clone, Debug)]
pub struct RaceFirstWithSleepToTimeoutMatch {
    pub source_file: Node,
    pub location: TextRange,
}


// Go: rules/race_first_with_sleep_to_timeout.go isRaceFirstTimerFlow

// Go: rules/race_first_with_sleep_to_timeout.go timerTrailingTransformations
