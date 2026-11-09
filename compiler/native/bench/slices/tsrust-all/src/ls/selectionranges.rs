use crate::ls::prelude::*;

// Go `internal/ls/selectionranges.go`: textDocument/selectionRange.

use crate::spanmap::Feature;
use std::cell::Cell;

// Go: ls/selectionranges.go:14 maxSelectionRangeDepth
const MAX_SELECTION_RANGE_DEPTH: usize = 1000;

// Go: ls/selectionranges.go:16 selectionRangeBuilder
// PORT: Go reads the capacity with `cap(b.ranges)`. A Rust `Vec` can hold
// more than it was asked for, so the capacity is a field.
struct SelectionRangeBuilder {
    ranges: Vec<lsproto::Range>,
    oldest_index: usize,
    capacity: usize,
}

// Go: ls/selectionranges.go:21 newSelectionRangeBuilder
fn new_selection_range_builder(capacity: usize) -> SelectionRangeBuilder {
    SelectionRangeBuilder {
        ranges: Vec::with_capacity(capacity),
        oldest_index: 0,
        capacity,
    }
}

impl SelectionRangeBuilder {
    // Go: ls/selectionranges.go:27 push
    fn push(&mut self, selection_range: lsproto::Range) {
        if self.ranges.len() < self.capacity {
            self.ranges.push(selection_range);
            return;
        }

        self.ranges[self.oldest_index] = selection_range;
        self.oldest_index = (self.oldest_index + 1) % self.ranges.len();
    }

    // Go: ls/selectionranges.go:37 build
    fn build(
        &self,
        mut result: Option<lsproto::SelectionRange>,
    ) -> Option<lsproto::SelectionRange> {
        for i in 0..self.ranges.len() {
            let index = (self.oldest_index + i) % self.ranges.len();
            result = Some(lsproto::SelectionRange {
                range: self.ranges[index],
                parent: result.map(Box::new),
            });
        }
        result
    }
}


// Go: ls/selectionranges.go:69 getSelectionChildren

// Go: ls/selectionranges.go:114 groupChildren

// Go: ls/selectionranges.go:134 splitChildren

// Go: ls/selectionranges.go:179 createSyntaxList

// Go: ls/selectionranges.go:185 getSmartSelectionRange
// PORT: Go builds the `*lsproto.SelectionRange` chain in `ranges.build`. The
// closures share `ranges`, `last_range` and `next` through `RefCell` and
// `Cell`, as Go closures share the locals. Go returns nil for a
// content-mapped file with no ranges; `None` is that nil.
