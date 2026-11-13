//! Port of Effect-TS/tsgo `internal/typeparser/piping_flow.go`.

use crate::effect::typeparser::*;
use crate::prelude::*;

/// Go `TransformationKind` (a Go `string` type).
/// TransformationKind represents how a transformation was expressed in source code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TransformationKind {
    /// Go `TransformationKindPipe` ("pipe").
    Pipe,
    /// Go `TransformationKindPipeable` ("pipeable").
    Pipeable,
    /// Go `TransformationKindDataFirst` ("dataFirst").
    DataFirst,
    /// Go `TransformationKindDataLast` ("dataLast").
    DataLast,
    /// Go `TransformationKindCall` ("call").
    Call,
    /// Go `TransformationKindEffectFn` ("effectFn").
    EffectFn,
    /// Go `TransformationKindEffectFnUntraced` ("effectFnUntraced").
    EffectFnUntraced,
}

impl TransformationKind {
    /// The Go string value of the kind.
    pub fn as_str(self) -> &'static str {
        match self {
            TransformationKind::Pipe => "pipe",
            TransformationKind::Pipeable => "pipeable",
            TransformationKind::DataFirst => "dataFirst",
            TransformationKind::DataLast => "dataLast",
            TransformationKind::Call => "call",
            TransformationKind::EffectFn => "effectFn",
            TransformationKind::EffectFnUntraced => "effectFnUntraced",
        }
    }
}









// Go: typeparser/piping_flow.go appliedCallOf

// Go: typeparser/piping_flow.go pipingFlowTransformationCall




// PORT: Go defines these `*PipingFlow` methods only to accept a nil
// receiver. A Rust `PipingFlow` is never nil (Go nil is `None` of
// `Option<Rc<PipingFlow>>`), so each forwards to the embedded flow.


/// Go `parsedSingleArgCallResult`.
/// parsedSingleArgCallResult is the internal result of parsing a single-argument call.
#[derive(Clone, Debug)]
pub struct ParsedSingleArgCallResult {
    /// the applied call
    pub node: Node,
    /// the applied function, decomposed for curried factories
    pub callee: Node,
    /// curried factory arguments, or nil
    pub args: Vec<Node>,
    /// the node carrying the transformation type arguments
    pub type_arguments_src: Node,
    pub subject: Node,
}







// Go: typeparser/piping_flow.go callTypeArguments

// Go: typeparser/piping_flow.go pipingFlowTypeArguments

