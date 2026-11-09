use crate::ls::prelude::*;

// Go `internal/ls/format.go`: the formatting requests and
// `getRangeOfEnclosingComment`.
//
// PORT: Go package `format` is written `crate::format` here, because this
// file is the module `ls::format`.

use crate::frontend::core_textchange::TextChange;
use crate::spanmap;
use crate::spanmap::{Feature, SpanMap};


// Go: ls/format.go:107 mappedFormattingRange
#[derive(Clone, Debug)]
struct MappedFormattingRange {
    projection: Node,
    segment: spanmap::Segment,
    original_range: TextRange,
}

// Go: ls/format.go:130 nonOverlappingFormattingRanges
// nonOverlappingFormattingRanges chooses at most one formatting projection for each original-text position.
// Candidates are ordered by original start and then descending end, so a longer mapping wins when several
// mappings start together:
//
//	candidates:  [---------- A ----------)
//	             [---- B ----)
//	result:      [---------- A ----------)
//
// Since starts are ordered, a candidate can only overlap the end of the last accepted range. Its start is
// trimmed to that end, preserving any uncovered suffix:
//
//	candidates:  [------- A -------)
//	                    [------- B ----------)
//	result:      [------- A -------)[-- B' --)
//
// Fully covered candidates have no suffix and are discarded. The segment itself is retained so callers can
// translate a trimmed original range to the corresponding offset in its virtual projection.


// Go: ls/format.go:257 getRangeOfEnclosingComment
// Unlike the TS implementation, this function *will not* compute default values for
// `precedingToken` and `tokenAtPosition`.
// It is the caller's responsibility to call `astnav.GetTokenAtPosition` to compute a default `tokenAtPosition`,
// or `astnav.FindPrecedingToken` to compute a default `precedingToken`.

// Go: ls/format_test.go
#[cfg(test)]
mod tests {
    use super::*;

    // Go: ls/format_test.go:15 TestNonOverlappingFormattingRanges
    #[test]
    fn test_non_overlapping_formatting_ranges() {
        let tests: &[(&str, &[TextRange], &[TextRange])] = &[
            (
                "sorts disjoint ranges",
                &[TextRange::new(10, 15), TextRange::new(0, 5)],
                &[TextRange::new(0, 5), TextRange::new(10, 15)],
            ),
            (
                "prefers longest range with same start",
                &[TextRange::new(0, 10), TextRange::new(0, 20)],
                &[TextRange::new(0, 20)],
            ),
            (
                "discards fully covered range",
                &[TextRange::new(5, 15), TextRange::new(0, 20)],
                &[TextRange::new(0, 20)],
            ),
            (
                "trims overlapping prefix",
                &[TextRange::new(5, 15), TextRange::new(0, 10)],
                &[TextRange::new(0, 10), TextRange::new(10, 15)],
            ),
        ];
        for (name, candidates, want) in tests {
            let candidates: Vec<MappedFormattingRange> = candidates
                .iter()
                .map(|&r| MappedFormattingRange {
                    projection: Node::NIL,
                    segment: spanmap::Segment::default(),
                    original_range: r,
                })
                .collect();
            let result: Vec<TextRange> = non_overlapping_formatting_ranges(&candidates)
                .into_iter()
                .map(|r| r.original_range)
                .collect();
            assert_eq!(&result, want, "{name}");
        }
    }
}
