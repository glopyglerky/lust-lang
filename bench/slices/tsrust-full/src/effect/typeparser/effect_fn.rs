//! Port of Effect-TS/tsgo `internal/typeparser/effect_fn.go`.

use crate::effect::typeparser::*;
use crate::prelude::*;

// PORT: Go `type EffectFnVariant string`. `as_str` gives the Go string.
// Go: typeparser/effect_fn.go EffectFnVariant
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EffectFnVariant {
    Fn,
    FnUntraced,
    FnUntracedEager,
}

impl EffectFnVariant {
    /// The Go string value of the variant.
    pub const fn as_str(self) -> &'static str {
        match self {
            EffectFnVariant::Fn => "fn",
            EffectFnVariant::FnUntraced => "fnUntraced",
            EffectFnVariant::FnUntracedEager => "fnUntracedEager",
        }
    }
}

impl std::fmt::Display for EffectFnVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}



// Go: typeparser/effect_fn.go splitEffectFnArguments

// Go: typeparser/effect_fn.go isGeneratorFunctionNode

