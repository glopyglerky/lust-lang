use crate::format::prelude::*;

use crate::flags_macros::go_enum;
use std::sync::{Arc, LazyLock};

// Go: format/rulesmap.go:11 getRules
// PORT: Go appends to the caller's slice and returns it. The caller (span.go)
// reuses one slice, so this appends to `rules` in place.

// Go: format/rulesmap.go:34 getRuleBucketIndex

// Go: format/rulesmap.go:40 maskBitSize
pub const MASK_BIT_SIZE: i32 = 5;
// Go: format/rulesmap.go:41 mask
pub const MASK: i32 = 0b11111; // MaskBitSize bits
// Go: format/rulesmap.go:42 mapRowLength
pub const MAP_ROW_LENGTH: i32 = SyntaxKind::LAST_TOKEN as i32 + 1;

/**
 * For a given rule action, gets a mask of other rule actions that
 * cannot be applied at the same position.
 */
// Go: format/rulesmap.go:49 getRuleActionExclusion
pub fn get_rule_action_exclusion(rule_action: RuleAction) -> RuleAction {
    let mut mask = RuleAction::NONE;
    if rule_action.intersects(RuleAction::STOP_PROCESSING_SPACE_ACTIONS) {
        mask |= RuleAction::MODIFY_SPACE_ACTION;
    }
    if rule_action.intersects(RuleAction::STOP_PROCESSING_TOKEN_ACTIONS) {
        mask |= RuleAction::MODIFY_TOKEN_ACTION;
    }
    if rule_action.intersects(RuleAction::MODIFY_SPACE_ACTION) {
        mask |= RuleAction::MODIFY_SPACE_ACTION;
    }
    if rule_action.intersects(RuleAction::MODIFY_TOKEN_ACTION) {
        mask |= RuleAction::MODIFY_TOKEN_ACTION;
    }
    mask
}

// Go: format/rulesmap.go:66 getRulesMap
// PORT: Go `sync.OnceValue(buildRulesMap)` is a process-wide LazyLock.

// Go: format/rulesmap.go:68 buildRulesMap

// Go: format/rulesmap.go:87 RulesPosition
go_enum!(RulesPosition, i32 {
    STOP_RULES_SPECIFIC = 0; // RulesPositionStopRulesSpecific
    STOP_RULES_ANY = MASK_BIT_SIZE * 1; // RulesPositionStopRulesAny
    CONTEXT_RULES_SPECIFIC = MASK_BIT_SIZE * 2; // RulesPositionContextRulesSpecific
    CONTEXT_RULES_ANY = MASK_BIT_SIZE * 3; // RulesPositionContextRulesAny
    NO_CONTEXT_RULES_SPECIFIC = MASK_BIT_SIZE * 4; // RulesPositionNoContextRulesSpecific
    NO_CONTEXT_RULES_ANY = MASK_BIT_SIZE * 5; // RulesPositionNoContextRulesAny
});

// The Rules list contains all the inserted rules into a rulebucket in the following order:
//
//	1- Ignore rules with specific token combination
//	2- Ignore rules with any token combination
//	3- Context rules with specific token combination
//	4- Context rules with any token combination
//	5- Non-context rules with specific token combination
//	6- Non-context rules with any token combination
//
// The member rulesInsertionIndexBitmap is used to describe the number of rules
// in each sub-bucket (above) hence can be used to know the index of where to insert
// the next rule. It's a bitmap which contains 6 different sections each is given 5 bits.
//
// Example:
// In order to insert a rule to the end of sub-bucket (3), we get the index by adding
// the values in the bitmap segments 3rd, 2nd, and 1st.
// Go: format/rulesmap.go:114 addRule
// PORT: Go returns the grown slice; here the bucket is updated in place.

// Go: format/rulesmap.go:143 getRuleInsertionIndex
pub fn get_rule_insertion_index(mut index_bitmap: i32, mask_position: RulesPosition) -> i32 {
    let mut index = 0;
    let mut pos = 0;
    while pos <= mask_position.0 {
        index += index_bitmap & MASK;
        index_bitmap >>= MASK_BIT_SIZE;
        pos += MASK_BIT_SIZE;
    }
    index
}

// Go: format/rulesmap.go:152 increaseInsertionIndex
