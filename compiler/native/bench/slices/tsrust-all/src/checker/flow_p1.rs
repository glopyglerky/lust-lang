//! Port of `checker/flow.go` lines 1-952: flow state, the flow graph walk
//! (`getTypeAtFlowNode`) and the narrowing functions up to
//! `getNarrowedTypeWorker`.

use crate::prelude::*;
use std::borrow::Cow;
use std::sync::LazyLock;

// Go: checker/flow.go:19 FlowType
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FlowType {
    pub t: TypeId,
    pub incomplete: bool,
}

impl FlowType {
    // Go: checker/flow.go:24 isNil
    #[must_use]
    pub fn is_nil(&self) -> bool {
        self.t.is_nil()
    }
}


// Go: checker/flow.go:35 SharedFlow
#[derive(Clone, Copy, Debug, Default)]
pub struct SharedFlow {
    pub flow: FlowNodeId,
    pub flow_type: FlowType,
}

/// Go `*ast.FlowReduceLabelData` on the flow state: the target label and
/// the temporary antecedent list of a reduce label flow node.
// PERF: the parts come from the flow node's `antecedents`
// (`[target, antecedents...]`, see `FlowNode::as_flow_reduce_label_data`).
// The list is a copy: a flow node of a freeable file version (lsshells M3b)
// is not `'static`, so the flow state cannot borrow it. Reduce labels come
// only from `finally` blocks; `get_branch_label_antecedents` still borrows
// the list of an ordinary branch label (perf14).
#[derive(Clone, Debug)]
pub struct ReduceLabel {
    pub target: FlowNodeId,
    pub antecedents: Box<[FlowNodeId]>,
}

impl ReduceLabel {
}

// Go: checker/flow.go:40 FlowState
// PORT: Go `*FlowState` is `Rc<RefCell<FlowState>>` (see `Checker::free_flow_state`).
// Go `[]*ast.FlowReduceLabelData` is a list of `ReduceLabel`.


// Go: checker/flow.go:69 getFlowNodeOfNode


// Go: checker/flow.go:208 getBranchLabelAntecedents
// PORT: Go returns the `*ast.FlowList`; here it is the antecedent slice of a
// flow node (see `core::FlowNode::antecedents` and `ReduceLabel`): borrowed
// from `flow_data`, or a copy of a reduce label's list (the caller changes
// the flow state while it walks the list).
// `flow_data` is `flow.get_flow()`, which the caller has read.
pub fn get_branch_label_antecedents<'a>(
    flow: FlowNodeId,
    flow_data: &'a FlowNode,
    reduce_labels: &[ReduceLabel],
) -> Cow<'a, [FlowNodeId]> {
    let mut i = reduce_labels.len();
    while i != 0 {
        i -= 1;
        let data = &reduce_labels[i];
        if data.target == flow {
            return Cow::Owned(data.antecedents.to_vec());
        }
    }
    Cow::Borrowed(&flow_data.antecedents)
}



// Go: checker/flow.go:635 typeofNEFacts
// PORT: Go package map var; read with `TYPEOF_NE_FACTS.get(name)`.
pub static TYPEOF_NE_FACTS: LazyLock<FxHashMap<&'static str, TypeFacts>> = LazyLock::new(|| {
    let mut m = FxHashMap::default();
    m.insert("string", TypeFacts::TYPEOF_NE_STRING);
    m.insert("number", TypeFacts::TYPEOF_NE_NUMBER);
    m.insert("bigint", TypeFacts::TYPEOF_NE_BIG_INT);
    m.insert("boolean", TypeFacts::TYPEOF_NE_BOOLEAN);
    m.insert("symbol", TypeFacts::TYPEOF_NE_SYMBOL);
    m.insert("undefined", TypeFacts::NE_UNDEFINED);
    m.insert("object", TypeFacts::TYPEOF_NE_OBJECT);
    m.insert("function", TypeFacts::TYPEOF_NE_FUNCTION);
    m
});

