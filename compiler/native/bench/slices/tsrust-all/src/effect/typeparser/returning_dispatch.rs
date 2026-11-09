//! Port of Effect-TS/tsgo `internal/typeparser/returning_dispatch.go`.

use crate::effect::typeparser::*;
use crate::prelude::*;

/// Go `DispatchConditionKind`: identifies how one result-producing branch is
/// selected.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum DispatchConditionKind {
    /// Go `DispatchConditionPredicate`: a boolean condition from an if or
    /// conditional expression.
    Predicate = 0,
    /// Go `DispatchConditionSwitchCase`: compares Subject, the switch
    /// discriminant, with Value, the case expression.
    SwitchCase = 1,
}

/// Go `DispatchCondition`: preserves the source nodes that select one
/// branch. Predicate conditions set Subject to the condition expression and
/// leave Value nil. Switch cases set Subject to the shared discriminant and
/// Value to the case expression. Source is the predicate expression or case
/// clause used for diagnostics. TagSubject and TagValue are populated when
/// the condition has the syntactic shape of an equality dispatch on a `_tag`
/// property.
#[derive(Clone, Copy, Debug)]
pub struct DispatchCondition {
    pub kind: DispatchConditionKind,
    pub source: Node,
    pub subject: Node,
    pub value: Node,
    pub tag_subject: Node,
    pub tag_value: Node,
}

/// Go `ResultDispatchBranch`: one source-ordered branch in a decoded
/// result-producing expression or block.
#[derive(Clone, Copy, Debug)]
pub struct ResultDispatchBranch {
    pub condition: DispatchCondition,
    pub result: Node,
}

/// Go `ResultDispatch`: an ordered, first-match result dispatch with an
/// optional fallback.
#[derive(Clone, Debug)]
pub struct ResultDispatch {
    pub node: Node,
    pub branches: Vec<ResultDispatchBranch>,
    pub fallback: Node,
}

/// Go `ParsedReturningDispatch`: a decoded arrow function or function
/// expression whose body consists entirely of result-producing branches.
/// PORT: Go `Dispatch` is a `*ResultDispatch` that `ParseReturningDispatch`
/// always sets.
#[derive(Clone, Debug)]
pub struct ParsedReturningDispatch {
    pub node: Node,
    pub params: Vec<Node>,
    pub body: Node,
    pub dispatch: Rc<ResultDispatch>,
}

/// Go `resultDispatchSyntax`.
/// PORT: the Go `*resultDispatchSyntax` results of this file are
/// `Option<ResultDispatchSyntax>`; no caller shares them.
#[derive(Clone, Debug, Default)]
pub struct ResultDispatchSyntax {
    pub branches: Vec<ResultDispatchBranch>,
    pub fallback: Node,
}















