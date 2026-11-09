//! Port of Go `checker/checker.go` lines 8445-9354: call, new, tagged
//! template, decorator and instanceof resolution, overload selection
//! (`resolveCall`, `chooseOverload`), arity checks and signature
//! applicability.

use crate::prelude::*;
use crate::smallvec::SmallVec;

// PORT: Go `*Relation` is shared and mutated across calls, so it is
// `Rc<RefCell<Relation>>`; a `*Relation` param is `&Rc<RefCell<Relation>>`.
// PORT: Go `candidatesOutArray *[]*Signature` is `Option<&mut Vec<SignatureId>>`.
// PORT: Go `diagnosticOutput *[]*ast.Diagnostic` is `Option<&mut Vec<Diagnostic>>`.
// PORT: a nullable Go `*diagnostics.Message` is `Option<&'static crate::diagnostics::Message>`.

// Go: checker/checker.go:9014 CallState
// PERF: callcopy1. `type_arguments` and `args` hold most lists inline, so a
// call copies each list once and allocates only for a long one. Go shares
// `node.TypeArguments()` and `node.Arguments()`; the port needs a `[Node]`
// for the callees that take `&[Node]`.

// Go: checker/checker.go:8829 constructorAccessibilityError
#[derive(Clone, Copy, Debug)]
pub struct ConstructorAccessibilityError {
    pub kind: ModifierFlags,
    pub declaring_class: TypeId,
}


