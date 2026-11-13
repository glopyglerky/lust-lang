//! Port of Go `ls/change/trackerimpl.go`.

use crate::ls::change::prelude::*;

use crate::frontend::scanner::scanner_p1::{};
use crate::frontend::scanner::{};
use crate::spanmap::{Feature, SpanMap};

// Go: ls/change/trackerimpl.go:91 dedupeIdenticalEdits

// Go: ls/change/trackerimpl.go:102 textEditsConflict
fn text_edits_conflict(
    a: &lsproto::TextEdit,
    b: &lsproto::TextEdit,
    multiple_projections: bool,
) -> bool {
    if lsproto::compare_positions(a.range.end, b.range.start) > 0 {
        return true;
    }
    // Different insertions at the same position are ambiguous when they may come from different projections.
    multiple_projections
        && a.range.start == a.range.end
        && a.range == b.range
        && a.new_text != b.new_text
}

// Go: ls/change/trackerimpl.go:222 leadingIndentation
// PORT: Go slices the string by bytes; this takes and returns bytes.
fn leading_indentation(text: &[u8]) -> &[u8] {
    let mut end = 0;
    while end < text.len() && (text[end] == b' ' || text[end] == b'\t') {
        end += 1;
    }
    &text[..end]
}


// Go: ls/change/trackerimpl.go:252 GetFormatCodeSettingsForWriting


// ============= utilities =============

// Go: ls/change/trackerimpl.go:404 hasCommentsBeforeLineBreak

// Go: ls/change/trackerimpl.go:413 needSemicolonBetween


// Go: ls/change/trackerimpl_test.go
#[cfg(test)]
mod tests {
    use super::*;

    // Go: ls/change/trackerimpl_test.go:9 TestTextEditsConflictAtSameInsertionPointAcrossProjections
    #[test]
    fn test_text_edits_conflict_at_same_insertion_point_across_projections() {
        let position = lsproto::Position {
            line: 1,
            character: 2,
        };
        let a = lsproto::TextEdit {
            range: lsproto::Range {
                start: position,
                end: position,
            },
            new_text: "a".to_string(),
        };
        let b = lsproto::TextEdit {
            range: lsproto::Range {
                start: position,
                end: position,
            },
            new_text: "b".to_string(),
        };

        assert!(
            text_edits_conflict(&a, &b, true),
            "different insertions at the same position across projections should conflict"
        );
        assert!(
            !text_edits_conflict(&a, &b, false),
            "insertions at the same position within one projection should retain their existing ordering"
        );
    }
}
