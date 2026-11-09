//! Port of Effect-TS/tsgo `internal/rules/prefer_succeed_some_or_none.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// PreferSucceedSomeOrNone suggests Effect.succeedNone and Effect.succeedSome for
// Effect.succeed calls that directly wrap Option.none or Option.some.


// PreferSucceedSomeOrNoneMatch holds the nodes needed by the diagnostic and quick fix.


// AnalyzePreferSucceedSomeOrNone finds piping flows in which Option.none or
// Option.some feeds directly into Effect.succeed. This covers both nested calls
// and pipe forms such as Option.none().pipe(Effect.succeed).


