//! Port of Go `ls/lsconv/linemap.go`.

use crate::ls::lsconv::prelude::*;

use crate::frontend::scanner::scanner_p1::{RUNE_SELF, };
use crate::gostd;

// Go: ls/lsconv/linemap.go:12 LSPLineStarts
// PORT: Go `[]core.TextPos`; `core.TextPos` is `int32`.
pub type LSPLineStarts = Vec<i32>;

// Go: ls/lsconv/linemap.go:14 LSPLineMap
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LSPLineMap {
    pub line_starts: LSPLineStarts,
    pub ascii_only: bool, // TODO(jakebailey): collect ascii-only info per line
    /// PORT: whether the text holds a marker unit (see
    /// `scanner_util::GO_STRING_MARKER`). Without one, port byte offsets
    /// are Go's, so a UTF-8 column needs no scan of its line (Go
    /// `converters.go` `start+char` and `position - start`).
    pub has_marker: bool,
}

// Go: ls/lsconv/linemap.go:19 ComputeLSPLineStarts
// PORT: Go returns `*LSPLineMap`, which callers share (file line map caches,
// `Converters.getLineMap`); here `Rc<LSPLineMap>`.

impl LSPLineMap {
    // Go: ls/lsconv/linemap.go:56 ComputeIndexOfLineStart
    #[must_use]
    pub fn compute_index_of_line_start(&self, target_pos: i32) -> i32 {
        // port of computeLineOfPosition(lineStarts: readonly number[], position: number, lowerBound?: number): number {
        let (line_number, ok) =
            gostd::slices::binary_search_func(&self.line_starts, target_pos, |p: &i32, t: &i32| {
                // Go: cmp.Compare(int(p), int(t))
                p.cmp(t) as i32
            });
        let mut line_number = line_number as i32;
        if !ok && line_number > 0 {
            // If the actual position was not found, the binary search returns where the target line start would be inserted
            // if the target was in the slice.
            // e.g. if the line starts at [5, 10, 23, 80] and the position requested was 20
            // then the search will return (3, false).
            //
            // We want the index of the previous line start, so we subtract 1.
            line_number -= 1;
        }
        line_number
    }
}
