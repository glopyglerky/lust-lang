//! Kernel extraction of the ts-rust core items the scanner uses:
//! core.TextRange (from ast/node.rs, where the port keeps it) and
//! the args! macro (from core.rs), copied verbatim.

// ──────────────────────────────────────────────────────────────────────
// core.TextRange (internal/core/text.go)
// ──────────────────────────────────────────────────────────────────────

/// Go `core.TextRange`. Positions are Go positions (UTF-8 byte offsets).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TextRange {
    pos: i32,
    end: i32,
}

impl TextRange {
    // Go: core/text.go:14 NewTextRange
    #[must_use]
    pub const fn new(pos: i32, end: i32) -> Self {
        Self { pos, end }
    }

    // Go: core/text.go:18 UndefinedTextRange
    #[must_use]
    pub const fn undefined() -> Self {
        Self { pos: -1, end: -1 }
    }

    // Go: core/text.go:22 Pos
    #[must_use]
    pub const fn pos(self) -> i32 {
        self.pos
    }

    // Go: core/text.go:26 End
    #[must_use]
    pub const fn end(self) -> i32 {
        self.end
    }

    // Go: core/text.go:30 Len
    #[must_use]
    pub const fn len(self) -> i32 {
        self.end - self.pos
    }

    // Go: core/text.go:34 IsValid
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.pos >= 0 || self.end >= 0
    }

    // Go: core/text.go:38 Contains
    #[must_use]
    pub const fn contains(self, pos: i32) -> bool {
        pos >= self.pos && pos < self.end
    }

    // Go: core/text.go:42 ContainsInclusive
    #[must_use]
    pub const fn contains_inclusive(self, pos: i32) -> bool {
        pos >= self.pos && pos <= self.end
    }

    // Go: core/text.go:46 ContainsExclusive
    #[must_use]
    pub const fn contains_exclusive(self, pos: i32) -> bool {
        self.pos < pos && pos < self.end
    }

    // Go: core/text.go:50 WithPos
    #[must_use]
    pub const fn with_pos(self, pos: i32) -> Self {
        Self { pos, end: self.end }
    }

    // Go: core/text.go:54 WithEnd
    #[must_use]
    pub const fn with_end(self, end: i32) -> Self {
        Self { pos: self.pos, end }
    }

    // Go: core/text.go:58 ContainedBy
    #[must_use]
    pub const fn contained_by(self, t2: TextRange) -> bool {
        t2.pos <= self.pos && t2.end >= self.end
    }

    // Go: core/text.go:62 Overlaps
    #[must_use]
    pub fn overlaps(self, t2: TextRange) -> bool {
        let start = self.pos.max(t2.pos);
        let end = self.end.min(t2.end);
        start < end
    }

    // Go: core/text.go:70 Intersects
    /// Like `overlaps`, but touching ranges intersect.
    #[must_use]
    pub fn intersects(self, t2: TextRange) -> bool {
        let start = self.pos.max(t2.pos);
        let end = self.end.min(t2.end);
        start <= end
    }
}

// Go: core/text.go:76 CompareTextRanges
#[must_use]
pub fn compare_text_ranges(r1: TextRange, r2: TextRange) -> i32 {
    let c = r1.pos - r2.pos;
    if c != 0 {
        return c;
    }
    r1.end - r2.end
}

/// A astdata range as a Go `core.TextRange`.
fn text_range_of(r: &crate::astdata::text::TextRange) -> TextRange {
    TextRange::new(r.start.get() as i32, r.end.get() as i32)
}



/// Go `...any` diagnostic arguments. Go formats each with `%v`; we use
/// `ToString`. Example: `args![self.type_to_string(t), count]`.
#[macro_export]
macro_rules! args {
    ($($arg:expr),* $(,)?) => { vec![$(::std::string::ToString::to_string(&$arg)),*] };
}

