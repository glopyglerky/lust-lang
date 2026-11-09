//! Port of internal/lsp/lsproto/util.go.

use crate::lsp::lsproto::prelude::*;

// Go: util.go:11 ComparePositions
// Implements a cmp.Compare like function for two Position
// ComparePositions(pos, other) == cmp.Compare(pos, other)
pub fn compare_positions(pos: Position, other: Position) -> i32 {
    let line_comp = pos.line.cmp(&other.line) as i32;
    if line_comp != 0 {
        return line_comp;
    }
    pos.character.cmp(&other.character) as i32
}

// Go: util.go:22 CompareRanges
// Implements a cmp.Compare like function for two Range
// CompareRanges(lsRange, other) == cmp.Compare(lsRange, other)
//
//	Range.Start is compared before Range.End
pub fn compare_ranges(ls_range: Range, other: Range) -> i32 {
    let start_comp = compare_positions(ls_range.start, other.start);
    if start_comp != 0 {
        return start_comp;
    }
    compare_positions(ls_range.end, other.end)
}


impl IntegerOrString {
    // Go: util.go:41 AsString
    pub fn as_string(&self) -> String {
        if let Some(s) = &self.string {
            s.clone()
        } else if let Some(i) = self.integer {
            i.to_string()
        } else {
            "-1".to_string()
        }
    }
}

// Go: util.go:53 diagnosticExistsInSlice

// Go: util.go:62 diagnosticsEqual

// Go: util.go:69 diagnosticCodesEqual
// PORT: a nil `*IntegerOrString` is a Go nil pointer dereference where Go
// reads it: `code1` first, `code2` only after a `code1` field is set.
fn diagnostic_codes_equal(
    code1: Option<&IntegerOrString>,
    code2: Option<&IntegerOrString>,
) -> bool {
    let code1 = code1.unwrap_or_else(|| crate::core::go_nil_dereference());
    if let Some(s1) = &code1.string {
        let code2 = code2.unwrap_or_else(|| crate::core::go_nil_dereference());
        if let Some(s2) = &code2.string {
            return s1 == s2;
        }
    }
    if let Some(i1) = code1.integer {
        let code2 = code2.unwrap_or_else(|| crate::core::go_nil_dereference());
        if let Some(i2) = code2.integer {
            return i1 == i2;
        }
    }
    false
}

// Go: util.go:79 diagnosticMessagesEqual

// Go: util.go:89 CompareDiagnostics
// PORT: Go takes and returns `[]*Diagnostic`; the results borrow from the inputs.

