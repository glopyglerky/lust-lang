// Go: internal/typeparser/effect_context.go

use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: `1 << iota` starts at iota 1 here, so CanYieldEffect is 2.
crate::flags_macros::go_flags!(EffectContextFlags, u8 {
    NONE = 0; // EffectContextFlagNone
    CAN_YIELD_EFFECT = 1 << 1; // EffectContextFlagCanYieldEffect
    IN_EFFECT_CONSTRUCTOR_THUNK = 1 << 2; // EffectContextFlagInEffectConstructorThunk
    PENDING_NEXT_FUNCTION_IS_EFFECT_THUNK = 1 << 3; // EffectContextFlagPendingNextFunctionIsEffectThunk
    PENDING_NEXT_OBJECT_TRY_PROPERTY_IS_EFFECT_THUNK = 1 << 4; // EffectContextFlagPendingNextObjectTryPropertyIsEffectThunk
    IN_EFFECT = (1 << 1) | (1 << 2); // EffectContextFlagInEffect
});




/// The locals of Go `analyzeEffectContextForSourceFile` that its closures
/// share. PORT: Go's `walk` closure is `effect_context_walk`, and the
/// closures that only touch the pending stores are methods here.
pub struct EffectContextWalk {
    pub pending_enable_flags: FxHashMap<Node, EffectContextFlags>,
    pub pending_disable_flags: FxHashMap<Node, EffectContextFlags>,
    pub pending_flags_mask: EffectContextFlags,
    pub function_scope_reset_flags: EffectContextFlags,
}

impl EffectContextWalk {
    // Go: setPendingEnableFlags
    pub fn set_pending_enable_flags(&mut self, node: Node, flags: EffectContextFlags) {
        if node.is_nil() || flags == EffectContextFlags::NONE {
            return;
        }
        *self.pending_enable_flags.entry(node).or_default() |= flags;
    }

    // Go: setPendingDisableFlags
    pub fn set_pending_disable_flags(&mut self, node: Node, flags: EffectContextFlags) {
        if node.is_nil() || flags == EffectContextFlags::NONE {
            return;
        }
        *self.pending_disable_flags.entry(node).or_default() |= flags;
    }

    // Go: resetChildFunctionScopeFlags
    pub fn reset_child_function_scope_flags(&mut self, node: Node) -> bool {
        let flags = self.function_scope_reset_flags;
        self.set_pending_disable_flags(node, flags);
        false
    }

    // Go: resetPendingFlags
    pub fn reset_pending_flags(&mut self, child: Node) -> bool {
        let flags = self.pending_flags_mask;
        self.set_pending_disable_flags(child, flags);
        false
    }
}

// Go: transparentPendingExpression

// Go: the `walk` closure of analyzeEffectContextForSourceFile
