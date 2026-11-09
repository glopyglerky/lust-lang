//! Port of Go package `evaluator` (`internal/evaluator/evaluator.go`).
//! Constant evaluation of enum member initializers and template literal types.

use crate::jsnum::{Number, PseudoBigInt};
use crate::prelude::*;

// PORT: Go `evaluator.Result` is named `EvaluatorResult` here. A type named
// `Result` would be glob-exported through the prelude and shadow
// `std::result::Result` in every port file.
// PORT: Go `Value any` is `Option<LiteralValue>`. `None` is Go `nil`. The
// evaluator produces the same dynamic values as `LiteralType.value` (string,
// jsnum.Number, jsnum.PseudoBigInt, bool), and the checker stores these
// values in literal types, so both share `LiteralValue` from `checker/types.rs`.
// Go: evaluator/evaluator.go:11 Result

// Go: evaluator/evaluator.go:18 NewResult

// PORT: Go `Evaluator func(expr, location) Result` is a stored closure. The
// only Go caller (the checker) passes `c.evaluateEntity`, so the closure gets
// the checker as its first argument. Call it as
// `let ev = self.evaluate.clone(); ev(self, expr, location)`.
// Go: evaluator/evaluator.go:22 Evaluator

// Go: evaluator/evaluator.go:24 NewEvaluator

// PORT: body of the Go `evaluate` closure created in NewEvaluator.
// Go: evaluator/evaluator.go:26 NewEvaluator.evaluate

// Go: evaluator/evaluator.go:124 evaluateTemplateExpression

// PORT: Go takes `any`; a nil value panics in Go ("Unhandled case"). Callers
// with an `Option<LiteralValue>` unwrap first, which panics the same way.
// Go: evaluator/evaluator.go:142 AnyToString

// Go: evaluator/evaluator.go:156 IsTruthy
