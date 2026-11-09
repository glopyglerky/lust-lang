//! Port of Effect-TS/tsgo `internal/rules/missed_pipeable_opportunity.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// MissedPipeableOpportunity detects nested function call chains that can be converted to .pipe() style.


// MissedPipeableOpportunityMatch holds the parsed result needed by both the diagnostic rule
// and the quick-fix for the missedPipeableOpportunity pattern.

// AnalyzeMissedPipeableOpportunity finds all nested call chains that can be converted to .pipe() style.

// isPipeableAtIndex checks if the type at a given index in a flow is pipeable.
// Index 0 = subject, index > 0 = transformations[index - 1].outType

// getSubjectText extracts the subject text for the diagnostic message.
// If the pipeable segment starts at index 0, uses the subject node text directly.
// Otherwise, traverses the flow node to find the node at the right depth.

// trimmedNodeText extracts the source text of a node, trimmed of leading trivia.
