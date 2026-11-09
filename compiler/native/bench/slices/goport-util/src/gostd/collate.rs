//! Go `golang.org/x/text/collate` v0.42.0 and the parts of
//! `golang.org/x/text/internal/colltab` that `Collator.CompareString` uses.
//! Its `tables.go` and `internal/colltab` are the same as in v0.38.0, and
//! `collate.go` differs only in a comment.
//! No port code calls it now. Go `ls/lsutil/organizeimports.go` used it at
//! the old default pin (dc37b5249ab6); from bump A (pin 52168999f3dc) Go
//! has its own unicode comparer, and Go N imports no `x/text/collate`. So
//! `unicode_is_ideographic` keeps its Unicode 15.0.0 copy (go1.27.1 has
//! Unicode 17.0.0): no output depends on it.
//!
//! PORT: Go's `tables.go` (CLDR 23, 95 locales) is dumped as little-endian
//! binary files in `data/` (`collate_main_*.bin`, 1.25 MB) by
//! `target/continuation-r97-goport/complete/gen/collate/gen.sh`
//! (zz_dump_test.go in a copy of x/text v0.38.0; the data is the same in
//! v0.42.0). The small tables
//! (`availableLocales`, `locales`, `varTop`) are at the end of this file.
//! Do not edit the data by hand. `norm` is `gostd::norm` and the Go
//! `unicode` tables are `gostd::unicode_tables`.
//!
//! PORT: Go keeps verbatim `[]byte` and `string` copies of the lookup, scan
//! and append functions. Rust has one `&[u8]` version; a `&str` is passed as
//! its bytes.

use crate::gostd::norm::{self, U16s, U32s};
use crate::gostd::unicode_tables::{self, RangeTable};
use crate::locale::language::{self, ComposePart, Confidence, Tag};
use crate::prelude::*;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// internal/colltab/collelem.go
// ---------------------------------------------------------------------------

// Go: internal/colltab/collelem.go:18 Level
/// Level identifies the collation comparison level.
/// The primary level corresponds to the basic sorting of text.
/// The secondary level corresponds to accents and related linguistic elements.
/// The tertiary level corresponds to casing and related concepts.
/// The quaternary level is derived from the other levels by the
/// various algorithms for handling variable elements.
pub type Level = usize;

// Go: internal/colltab/collelem.go:20 (Level constants)
pub const PRIMARY: Level = 0;
pub const SECONDARY: Level = 1;
pub const TERTIARY: Level = 2;
pub const QUATERNARY: Level = 3;
pub const IDENTITY: Level = 4;
pub const NUM_LEVELS: usize = 5;

// Go: internal/colltab/collelem.go:30
const DEFAULT_SECONDARY: i32 = 0x20;
const DEFAULT_TERTIARY: i32 = 0x2;
const MAX_TERTIARY: u8 = 0x1F;
pub const MAX_QUATERNARY: i32 = 0x1FFFFF; // 21 bits.

// Go: internal/colltab/collelem.go:41 Elem
/// Elem is a representation of a collation element. This API provides ways to encode
/// and decode Elems. Implementations of collation tables may use values greater
/// or equal to PrivateUse for their own purposes.  However, these should never be
/// returned by AppendNext.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Elem(pub u32);

// Go: internal/colltab/collelem.go:43
const MAX_CE: u32 = 0xAFFFFFFF;
const MAX_CONTRACT: u32 = 0xDFFFFFFF;
const MAX_EXPAND: u32 = 0xEFFFFFFF;

// Go: internal/colltab/collelem.go:53 ceType
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CeType {
    /// ceNormal includes implicits (ce == 0)
    Normal,
    /// rune can be a start of a contraction
    ContractionIndex,
    /// rune expands into a sequence of collation elements
    ExpansionIndex,
    /// rune expands using NFKC decomposition
    Decompose,
}

// Go: internal/colltab/collelem.go:100
// For normal collation elements, we assume that a collation element either has
// a primary or non-default secondary value, not both.
// Collation elements with a primary value are of the form
//
//	01pppppp pppppppp ppppppp0 ssssssss
//	  - p* is primary collation value
//	  - s* is the secondary collation value
//	00pppppp pppppppp ppppppps sssttttt, where
//	  - p* is primary collation value
//	  - s* offset of secondary from default value.
//	  - t* is the tertiary collation value
//	100ttttt cccccccc pppppppp pppppppp
//	  - t* is the tertiar collation value
//	  - c* is the canonical combining class
//	  - p* is the primary collation value
//
// Collation elements with a secondary value are of the form
//
//	1010cccc ccccssss ssssssss tttttttt, where
//	  - c* is the canonical combining class
//	  - s* is the secondary collation value
//	  - t* is the tertiary collation value
//	11qqqqqq qqqqqqqq qqqqqqq0 00000000
//	  - q* quaternary value
const CE_TYPE_MASK: u32 = 0xC0000000;
const CE_TYPE_MASK_EXT: u32 = 0xE0000000;
const CE_IGNORE_MASK: u32 = 0xF00FFFFF;
const CE_TYPE1: u32 = 0x40000000;
const CE_TYPE2: u32 = 0x00000000;
const CE_TYPE3OR4: u32 = 0x80000000;
const CE_TYPE4: u32 = 0xA0000000;
const CE_TYPE_Q: u32 = 0xC0000000;
const IGNORE: u32 = CE_TYPE4;
const FIRST_NON_PRIMARY: u32 = 0x80000000;
const LAST_SPECIAL_PRIMARY: u32 = 0xA0000000;
const HAS_TERTIARY_MASK: u32 = 0x40000000;
const PRIMARY_VALUE_MASK: u32 = 0x3FFFFE00;
const MAX_PRIMARY_BITS: u32 = 21;
const COMPACT_PRIMARY_BITS: u32 = 16;
const MAX_SECONDARY_BITS: u32 = 12;
const MAX_TERTIARY_BITS: u32 = 8;
const MAX_CCC_BITS: u32 = 8;
const MAX_SECONDARY_COMPACT_BITS: u32 = 8;
const MAX_SECONDARY_DIFF_BITS: u32 = 4;
const MAX_TERTIARY_COMPACT_BITS: u32 = 5;
const PRIMARY_SHIFT: u32 = 9;
const COMPACT_SECONDARY_SHIFT: u32 = 5;
const MIN_COMPACT_SECONDARY: i32 = DEFAULT_SECONDARY - 4;

// Go: internal/colltab/collelem.go:128 makeImplicitCE
fn make_implicit_ce(primary: i32) -> Elem {
    Elem(CE_TYPE1 | ((primary as u32) << PRIMARY_SHIFT) | DEFAULT_SECONDARY as u32)
}

// Go: internal/colltab/collelem.go:134 MakeElem
/// MakeElem returns an Elem for the given values.  It will return an error
/// if the given combination of values is invalid.
pub fn make_elem(primary: i32, secondary: i32, tertiary: i32, ccc: u8) -> Result<Elem, String> {
    let w = primary;
    if w >= 1 << MAX_PRIMARY_BITS || w < 0 {
        return Err(format!(
            "makeCE: primary weight out of bounds: {:x} >= {:x}",
            w,
            1 << MAX_PRIMARY_BITS
        ));
    }
    let w = secondary;
    if w >= 1 << MAX_SECONDARY_BITS || w < 0 {
        return Err(format!(
            "makeCE: secondary weight out of bounds: {:x} >= {:x}",
            w,
            1 << MAX_SECONDARY_BITS
        ));
    }
    let w = tertiary;
    if w >= 1 << MAX_TERTIARY_BITS || w < 0 {
        return Err(format!(
            "makeCE: tertiary weight out of bounds: {:x} >= {:x}",
            w,
            1 << MAX_TERTIARY_BITS
        ));
    }
    let mut ce: u32;
    if primary != 0 {
        if ccc != 0 {
            if primary >= 1 << COMPACT_PRIMARY_BITS {
                return Err(format!(
                    "makeCE: primary weight with non-zero CCC out of bounds: {:x} >= {:x}",
                    primary,
                    1 << COMPACT_PRIMARY_BITS
                ));
            }
            if secondary != DEFAULT_SECONDARY {
                return Err(format!(
                    "makeCE: cannot combine non-default secondary value ({:x}) with non-zero CCC ({:x})",
                    secondary, ccc
                ));
            }
            ce = (tertiary as u32) << (COMPACT_PRIMARY_BITS + MAX_CCC_BITS);
            ce |= u32::from(ccc) << COMPACT_PRIMARY_BITS;
            ce |= primary as u32;
            ce |= CE_TYPE3OR4;
        } else if tertiary == DEFAULT_TERTIARY {
            if secondary >= 1 << MAX_SECONDARY_COMPACT_BITS {
                return Err(format!(
                    "makeCE: secondary weight with non-zero primary out of bounds: {:x} >= {:x}",
                    secondary,
                    1 << MAX_SECONDARY_COMPACT_BITS
                ));
            }
            ce = ((primary << (MAX_SECONDARY_COMPACT_BITS + 1)) + secondary) as u32;
            ce |= CE_TYPE1;
        } else {
            let d = secondary - DEFAULT_SECONDARY + MAX_SECONDARY_DIFF_BITS as i32;
            if d >= 1 << MAX_SECONDARY_DIFF_BITS || d < 0 {
                return Err(format!(
                    "makeCE: secondary weight diff out of bounds: {:x} < 0 || {:x} > {:x}",
                    d,
                    d,
                    1 << MAX_SECONDARY_DIFF_BITS
                ));
            }
            if tertiary >= 1 << MAX_TERTIARY_COMPACT_BITS {
                return Err(format!(
                    "makeCE: tertiary weight with non-zero primary out of bounds: {:x} > {:x}",
                    tertiary,
                    1 << MAX_TERTIARY_COMPACT_BITS
                ));
            }
            ce = ((primary << MAX_SECONDARY_DIFF_BITS) + d) as u32;
            ce = (ce << MAX_TERTIARY_COMPACT_BITS) + tertiary as u32;
        }
    } else {
        ce = ((secondary << MAX_TERTIARY_BITS) + tertiary) as u32;
        ce += u32::from(ccc) << (MAX_SECONDARY_BITS + MAX_TERTIARY_BITS);
        ce |= CE_TYPE4;
    }
    Ok(Elem(ce))
}

impl Elem {
    // Go: internal/colltab/collelem.go:62 (Elem).ctype
    fn ctype(self) -> CeType {
        let ce = self.0;
        if ce <= MAX_CE {
            return CeType::Normal;
        }
        if ce <= MAX_CONTRACT {
            CeType::ContractionIndex
        } else {
            if ce <= MAX_EXPAND {
                return CeType::ExpansionIndex;
            }
            CeType::Decompose
        }
    }

    // Go: internal/colltab/collelem.go:196 (Elem).CCC
    /// CCC returns the canonical combining class associated with the underlying character,
    /// if applicable, or 0 otherwise.
    pub fn ccc(self) -> u8 {
        let ce = self.0;
        if ce & CE_TYPE3OR4 != 0 {
            if ce & CE_TYPE4 == CE_TYPE3OR4 {
                return (ce >> 16) as u8;
            }
            return (ce >> 20) as u8;
        }
        0
    }

    // Go: internal/colltab/collelem.go:207 (Elem).Primary
    /// Primary returns the primary collation weight for ce.
    pub fn primary(self) -> i32 {
        let ce = self.0;
        if ce >= FIRST_NON_PRIMARY {
            if ce > LAST_SPECIAL_PRIMARY {
                return 0;
            }
            return i32::from(ce as u16);
        }
        ((ce & PRIMARY_VALUE_MASK) >> PRIMARY_SHIFT) as i32
    }

    // Go: internal/colltab/collelem.go:218 (Elem).Secondary
    /// Secondary returns the secondary collation weight for ce.
    pub fn secondary(self) -> i32 {
        let ce = self.0;
        match ce & CE_TYPE_MASK {
            CE_TYPE1 => i32::from(ce as u8),
            CE_TYPE2 => MIN_COMPACT_SECONDARY + ((ce >> COMPACT_SECONDARY_SHIFT) & 0xF) as i32,
            CE_TYPE3OR4 => {
                if ce < CE_TYPE4 {
                    return DEFAULT_SECONDARY;
                }
                ((ce >> 8) & 0xFFF) as i32
            }
            CE_TYPE_Q => 0,
            _ => panic!("should not reach here"),
        }
    }

    // Go: internal/colltab/collelem.go:236 (Elem).Tertiary
    /// Tertiary returns the tertiary collation weight for ce.
    pub fn tertiary(self) -> u8 {
        let ce = self.0;
        if ce & HAS_TERTIARY_MASK == 0 {
            if ce & CE_TYPE3OR4 == 0 {
                return (ce & 0x1F) as u8;
            }
            if ce & CE_TYPE4 == CE_TYPE4 {
                return ce as u8;
            }
            return ((ce >> 24) as u8) & 0x1F; // type 2
        } else if ce & CE_TYPE_MASK == CE_TYPE1 {
            return DEFAULT_TERTIARY as u8;
        }
        // ce is a quaternary value.
        0
    }

    // Go: internal/colltab/collelem.go:252 (Elem).updateTertiary
    fn update_tertiary(self, t: u8) -> Elem {
        let mut ce = self.0;
        if ce & CE_TYPE_MASK == CE_TYPE1 {
            // convert to type 4
            let mut nce = ce & PRIMARY_VALUE_MASK;
            nce |= u32::from((ce as u8).wrapping_sub(MIN_COMPACT_SECONDARY as u8))
                << COMPACT_SECONDARY_SHIFT;
            ce = nce;
        } else if ce & CE_TYPE_MASK_EXT == CE_TYPE3OR4 {
            ce &= !(u32::from(MAX_TERTIARY) << 24);
            return Elem(ce | (u32::from(t) << 24));
        } else {
            // type 2 or 4
            ce &= !u32::from(MAX_TERTIARY);
        }
        Elem(ce | u32::from(t))
    }

    // Go: internal/colltab/collelem.go:271 (Elem).Quaternary
    /// Quaternary returns the quaternary value if explicitly specified,
    /// 0 if ce == Ignore, or MaxQuaternary otherwise.
    /// Quaternary values are used only for shifted variants.
    pub fn quaternary(self) -> i32 {
        let ce = self.0;
        if ce & CE_TYPE_MASK == CE_TYPE_Q {
            return ((ce & PRIMARY_VALUE_MASK) >> PRIMARY_SHIFT) as i32;
        } else if ce & CE_IGNORE_MASK == IGNORE {
            return 0;
        }
        MAX_QUATERNARY
    }
}

// Go: internal/colltab/collelem.go:302
// For contractions, collation elements are of the form
// 110bbbbb bbbbbbbb iiiiiiii iiiinnnn, where
//   - n* is the size of the first node in the contraction trie.
//   - i* is the index of the first node in the contraction trie.
//   - b* is the offset into the contraction collation element table.
//
// See contract.go for details on the contraction trie.
const MAX_N_BITS: u32 = 4;
const MAX_TRIE_INDEX_BITS: u32 = 12;
const MAX_CONTRACT_OFFSET_BITS: u32 = 13;

// Go: internal/colltab/collelem.go:308 splitContractIndex
fn split_contract_index(ce: Elem) -> (usize, usize, usize) {
    let mut ce = ce.0;
    let n = (ce & ((1 << MAX_N_BITS) - 1)) as usize;
    ce >>= MAX_N_BITS;
    let index = (ce & ((1 << MAX_TRIE_INDEX_BITS) - 1)) as usize;
    ce >>= MAX_TRIE_INDEX_BITS;
    let offset = (ce & ((1 << MAX_CONTRACT_OFFSET_BITS) - 1)) as usize;
    (index, n, offset)
}

// Go: internal/colltab/collelem.go:321 splitExpandIndex
// For expansions, Elems are of the form 11100000 00000000 bbbbbbbb bbbbbbbb,
// where b* is the index into the expansion sequence table.
fn split_expand_index(ce: Elem) -> usize {
    usize::from(ce.0 as u16)
}

// Go: internal/colltab/collelem.go:334 splitDecompose
// Some runes can be expanded using NFKD decomposition. Instead of storing the full
// sequence of collation elements, we decompose the rune and lookup the collation
// elements for each rune in the decomposition and modify the tertiary weights.
// The Elem, in this case, is of the form 11110000 00000000 wwwwwwww vvvvvvvv, where
//   - v* is the replacement tertiary weight for the first rune,
//   - w* is the replacement tertiary weight for the second rune,
//
// Tertiary weights of subsequent runes should be replaced with maxTertiary.
// See https://www.unicode.org/reports/tr10/#Compatibility_Decompositions for more details.
fn split_decompose(ce: Elem) -> (u8, u8) {
    (ce.0 as u8, (ce.0 >> 8) as u8)
}

// Go: internal/colltab/collelem.go:338
// These constants were taken from https://www.unicode.org/versions/Unicode6.0.0/ch12.pdf.
const MIN_UNIFIED: u32 = 0x4E00;
const MAX_UNIFIED: u32 = 0x9FFF;
const MIN_COMPATIBILITY: u32 = 0xF900;
const MAX_COMPATIBILITY: u32 = 0xFAFF;

// Go: internal/colltab/collelem.go:347
const COMMON_UNIFIED_OFFSET: i32 = 0x10000;
const RARE_UNIFIED_OFFSET: i32 = 0x20000; // largest rune in common is U+FAFF
const OTHER_OFFSET: i32 = 0x50000; // largest rune in rare is U+2FA1D

// Go: internal/colltab/collelem.go:360 implicitPrimary
/// implicitPrimary returns the primary weight for the given rune
/// for which there is no entry for the rune in the collation table.
/// We take a different approach from the one specified in
/// https://unicode.org/reports/tr10/#Implicit_Weights,
/// but preserve the resulting relative ordering of the runes.
fn implicit_primary(r: u32) -> i32 {
    if unicode_is_ideographic(r) {
        if (MIN_UNIFIED..=MAX_UNIFIED).contains(&r) {
            // The most common case for CJK.
            return r as i32 + COMMON_UNIFIED_OFFSET;
        }
        if (MIN_COMPATIBILITY..=MAX_COMPATIBILITY).contains(&r) {
            // This will typically not hit. The DUCET explicitly specifies mappings
            // for all characters that do not decompose.
            return r as i32 + COMMON_UNIFIED_OFFSET;
        }
        return r as i32 + RARE_UNIFIED_OFFSET;
    }
    r as i32 + OTHER_OFFSET
}

/// Go `unicode.Is(unicode.Ideographic, r)` (Go 1.26, Unicode 15.0.0; no
/// caller now, see the module note).
fn unicode_is_ideographic(r: u32) -> bool {
    const IDEOGRAPHIC: [(u32, u32); 20] = [
        (0x3006, 0x3007),
        (0x3021, 0x3029),
        (0x3038, 0x303a),
        (0x3400, 0x4dbf),
        (0x4e00, 0x9fff),
        (0xf900, 0xfa6d),
        (0xfa70, 0xfad9),
        (0x16fe4, 0x16fe4),
        (0x17000, 0x187f7),
        (0x18800, 0x18cd5),
        (0x18d00, 0x18d08),
        (0x1b170, 0x1b2fb),
        (0x20000, 0x2a6df),
        (0x2a700, 0x2b739),
        (0x2b740, 0x2b81d),
        (0x2b820, 0x2cea1),
        (0x2ceb0, 0x2ebe0),
        (0x2f800, 0x2fa1d),
        (0x30000, 0x3134a),
        (0x31350, 0x323af),
    ];
    IDEOGRAPHIC.iter().any(|&(lo, hi)| (lo..=hi).contains(&r))
}

// ---------------------------------------------------------------------------
// internal/colltab/weighter.go
// ---------------------------------------------------------------------------

// Go: internal/colltab/weighter.go:8 Weighter
/// A Weighter can be used as a source for Collator and Searcher.
/// PORT: `Start`, `StartString` and `Domain` panic in Go's `Table` and are
/// not used by `Collator.CompareString`; they are not ported.
pub trait Weighter {
    /// AppendNext appends Elems to buf corresponding to the longest match
    /// of a single character or contraction from the start of s.
    /// It returns the new buf and the number of bytes consumed.
    fn append_next(&self, buf: Vec<Elem>, s: &[u8]) -> (Vec<Elem>, usize);

    /// Top returns the highest variable primary value.
    fn top(&self) -> u32;
}

// ---------------------------------------------------------------------------
// internal/colltab/trie.go
// ---------------------------------------------------------------------------

// The trie in this file is used to associate the first full character in an
// UTF-8 string to a collation element. All but the last byte in a UTF-8 byte
// sequence are used to lookup offsets in the index table to be used for the
// next byte. The last byte is used to index into a table of collation elements.
// For a full description, see go.text/collate/build/trie.go.

// Go: internal/colltab/trie.go:15 Trie
/// PORT: the fields are slices of the generated tables (see `get_table`).
pub struct Trie {
    /// index for first byte (0xC0-0xFF)
    pub index0: U16s,
    /// index for first byte (0x00-0x7F)
    pub values0: U32s,
    pub index: U16s,
    pub values: U32s,
}

// Go: internal/colltab/trie.go:22
const TX: u8 = 0x80; // 1000 0000
const T2: u8 = 0xC0; // 1100 0000
const T3: u8 = 0xE0; // 1110 0000
const T4: u8 = 0xF0; // 1111 0000
const T5: u8 = 0xF8; // 1111 1000

impl Trie {
    // Go: internal/colltab/trie.go:33 (*Trie).lookupValue

    // Go: internal/colltab/trie.go:40 (*Trie).lookup
}

// ---------------------------------------------------------------------------
// internal/colltab/contract.go
// ---------------------------------------------------------------------------

// Go: internal/colltab/contract.go:11 ContractTrieSet
// For a description of ContractTrieSet, see text/collate/build/contract.go.
#[derive(Clone, Copy, Debug)]
pub struct CtEntry {
    pub l: u8,
    pub h: u8,
    pub n: u8,
    pub i: u8,
}

/// PORT: Go `[]struct{ L, H, N, I uint8 }` stored as 4 bytes per entry.
pub struct ContractTrieSet(pub &'static [u8]);

impl ContractTrieSet {
    fn entry(&self, i: usize) -> CtEntry {
        let b = &self.0[4 * i..4 * i + 4];
        CtEntry {
            l: b[0],
            h: b[1],
            n: b[2],
            i: b[3],
        }
    }
}

// Go: internal/colltab/contract.go:21 ctScanner
/// ctScanner is used to match a trie to an input sequence.
/// A contraction may match a non-contiguous sequence of bytes in an input string.
/// For example, if there is a contraction for <a, combining_ring>, it should match
/// the sequence <a, combining_cedilla, combining_ring>, as combining_cedilla does
/// not block combining_ring.
/// ctScanner does not automatically skip over non-blocking non-starters, but rather
/// retains the state of the last match and leaves it up to the user to continue
/// the match at the appropriate points.
/// PORT: Go `states` is a slice of the set; here it is the offset of that
/// slice in `set`.
struct CtScanner<'a> {
    set: &'static ContractTrieSet,
    states: usize,
    s: &'a [u8],
    n: usize,
    index: usize,
    pindex: usize,
    done: bool,
}

impl ContractTrieSet {
    // Go: internal/colltab/contract.go:39 (ContractTrieSet).scanner
    fn scanner<'a>(&'static self, index: usize, n: usize, b: &'a [u8]) -> CtScanner<'a> {
        CtScanner {
            set: self,
            s: b,
            states: index,
            n,
            index: 0,
            pindex: 0,
            done: false,
        }
    }
}

// Go: internal/colltab/contract.go:57
const FINAL: u8 = 0;
const NO_INDEX: u8 = 0xFF;

impl CtScanner<'_> {
    // Go: internal/colltab/contract.go:49 (*ctScanner).result
    /// result returns the offset i and bytes consumed p so far.  If no suffix
    /// matched, i and p will be 0.
    fn result(&self) -> (usize, usize) {
        (self.index, self.pindex)
    }

    // Go: internal/colltab/contract.go:64 (*ctScanner).scan
    /// scan matches the longest suffix at the current location in the input
    /// and returns the number of bytes consumed.
    fn scan(&mut self, mut p: usize) -> usize {
        let mut pr = p; // the p at the rune start
        let str = self.s;
        let (mut states, mut n) = (self.states, self.n);
        let mut i = 0;
        while i < n && p < str.len() {
            let e = self.set.entry(states + i);
            let c = str[p];
            // TODO: a significant number of contractions are of a form that
            // cannot match discontiguous UTF-8 in a normalized string. We could let
            // a negative value of e.n mean that we can set s.done = true and avoid
            // the need for additional matches.
            if c >= e.l {
                if e.l == c {
                    p += 1;
                    if e.i != NO_INDEX {
                        self.index = usize::from(e.i);
                        self.pindex = p;
                    }
                    if e.n != FINAL {
                        (i, states, n) = (0, states + usize::from(e.h) + n, usize::from(e.n));
                        if p >= str.len() || utf8_rune_start(str[p]) {
                            (self.states, self.n, pr) = (states, n, p);
                        }
                    } else {
                        self.done = true;
                        return p;
                    }
                    continue;
                } else if e.n == FINAL && c <= e.h {
                    p += 1;
                    self.done = true;
                    self.index = usize::from(c - e.l) + usize::from(e.i);
                    self.pindex = p;
                    return p;
                }
            }
            i += 1;
        }
        pr
    }
}

// ---------------------------------------------------------------------------
// internal/colltab/table.go
// ---------------------------------------------------------------------------

// Go: internal/colltab/table.go:14 Table
/// Table holds all collation data for a given collation ordering.
pub struct Table {
    /// main trie
    pub index: Trie,

    // expansion info
    pub expand_elem: U32s,

    // contraction info
    pub contract_tries: &'static ContractTrieSet,
    pub contract_elem: U32s,
    pub max_contract_len: i32,
    pub variable_top: u32,
}


impl Table {
    // Go: internal/colltab/table.go:100 (*Table).appendNext

    // Go: internal/colltab/table.go:154 (*Table).appendExpansion

    // Go: internal/colltab/table.go:164 (*Table).matchContraction
    // Go: internal/colltab/table.go:222 (*Table).matchContractionString
}

// ---------------------------------------------------------------------------
// internal/colltab/iter.go
// ---------------------------------------------------------------------------

// Go: internal/colltab/iter.go:10 Iter
/// An Iter incrementally converts chunks of the input text to collation
/// elements, while ensuring that the collation elements are in normalized order
/// (that is, they are in the order as if the input text were normalized first).
/// PORT: Go keeps the input string; this keeps a reused copy of its bytes.
pub struct Iter {
    pub weighter: Rc<dyn Weighter>,
    pub elems: Vec<Elem>,
    /// N is the number of elements in Elems that will not be reordered on
    /// subsequent iterations, N <= len(Elems).
    pub n: usize,

    str: Vec<u8>,
    // Because the Elems buffer may contain collation elements that are needed
    // for look-ahead, we need two positions in the text (bytes or str): one for
    // the end position in the text for the current iteration and one for the
    // start of the next call to appendNext.
    /// end position in text corresponding to N.
    p_end: usize,
    /// pEnd <= pNext.
    p_next: usize,
}

// Go: internal/colltab/iter.go:163 maxCombiningCharacters
const MAX_COMBINING_CHARACTERS: usize = 30;

impl Iter {
    fn new(weighter: Rc<dyn Weighter>) -> Iter {
        Iter {
            weighter,
            elems: Vec::with_capacity(512),
            n: 0,
            str: Vec::new(),
            p_end: 0,
            p_next: 0,
        }
    }

    // Go: internal/colltab/iter.go:29 (*Iter).Reset
    /// Reset sets the position in the current input text to p and discards any
    /// results obtained so far.
    pub fn reset(&mut self, p: usize) {
        self.elems.clear();
        self.n = 0;
        self.p_end = p;
        self.p_next = p;
    }

    // Go: internal/colltab/iter.go:66 (*Iter).SetInputString
    /// SetInputString resets i to input s.
    pub fn set_input_string(&mut self, s: &str) {
        self.str.clear();
        self.str.extend_from_slice(s.as_bytes());
        self.reset(0);
    }

    // Go: internal/colltab/iter.go:72 (*Iter).done
    fn done(&self) -> bool {
        self.p_next >= self.str.len()
    }

    // Go: internal/colltab/iter.go:76 (*Iter).appendNext
    fn append_next(&mut self) -> bool {
        if self.done() {
            return false;
        }
        let elems = std::mem::take(&mut self.elems);
        let (elems, mut sz) = self.weighter.append_next(elems, &self.str[self.p_next..]);
        self.elems = elems;
        if sz == 0 {
            sz = 1;
        }
        self.p_next += sz;
        true
    }

    // Go: internal/colltab/iter.go:98 (*Iter).Next

    // Go: internal/colltab/iter.go:170 (*Iter).doNorm
}

// ---------------------------------------------------------------------------
// internal/colltab/numeric.go
// ---------------------------------------------------------------------------

// Go: internal/colltab/numeric.go:18 NewNumericWeighter

// Go: internal/colltab/numeric.go:48 numericWeighter
/// A numericWeighter translates a stream of digits into a stream of weights
/// representing the numeric value.
struct NumericWeighter {
    weighter: Rc<dyn Weighter>,

    // The Elems below all demarcate boundaries of specific ranges. With the
    // current element encoding digits are in two ranges: normal (default
    // tertiary value) and special. For most languages, digits have collation
    // elements in the normal range.
    //
    // Note: the range tests are very specific for the element encoding used by
    // this implementation. The tests in collate_test.go are designed to fail
    // if this code is not updated when an encoding has changed.
    /// normal digit zero
    zero: Elem,
    /// special digit zero, low tertiary value
    zero_special_lo: Elem,
    /// special digit zero, high tertiary value
    zero_special_hi: Elem,
    /// normal digit nine
    nine: Elem,
    /// special digit nine
    nine_special_hi: Elem,
    number_start: Elem,
}


// Go: internal/colltab/numeric.go:119 numberConverter
struct NumberConverter<'a> {
    w: &'a NumericWeighter,

    elems: Vec<Elem>,
    n_digits: i32,
    len_index: usize,

    /// PORT: Go `s` and `b`; the input bytes.
    s: &'a [u8],
}

// Go: internal/colltab/numeric.go:215 maxDigits

impl NumberConverter<'_> {
    // Go: internal/colltab/numeric.go:132 (*numberConverter).init

    // Go: internal/colltab/numeric.go:151 (*numberConverter).checkNextDigit

    // Go: internal/colltab/numeric.go:197 (*numberConverter).isDigit

    // Go: internal/colltab/numeric.go:217 (*numberConverter).update

    // Go: internal/colltab/numeric.go:232 (*numberConverter).result
    /// result fills in the length element for the digit sequence and returns the
    /// completed collation elements.
    fn result(mut self) -> Vec<Elem> {
        let e =
            make_elem(self.n_digits, DEFAULT_SECONDARY, DEFAULT_TERTIARY, 0).unwrap_or_default();
        self.elems[self.len_index] = e;
        self.elems
    }
}

// ---------------------------------------------------------------------------
// internal/colltab/colltab.go
// ---------------------------------------------------------------------------

// Go: internal/colltab/colltab.go:25 MatchLang

// Go: internal/colltab/colltab.go:91 parent

// Go: language/language.go:188 (CanonType).Canonicalize (with language.All)

// ---------------------------------------------------------------------------
// collate/index.go
// ---------------------------------------------------------------------------

// Go: collate/index.go:29 tableIndex
/// tableIndex holds information for constructing a table
/// for a certain locale based on the main table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TableIndex {
    lookup_offset: u32,
    values_offset: u32,
}

// Go: collate/index.go:9 blockSize
const BLOCK_SIZE: usize = 64;

// Go: collate/index.go:11 getTable

// ---------------------------------------------------------------------------
// collate/option.go
// ---------------------------------------------------------------------------

// Go: collate/option.go:221 alternateHandling
/// alternateHandling identifies the various ways in which variables are handled.
/// A rune with a primary weight lower than the variable top is considered a
/// variable.
/// See https://www.unicode.org/reports/tr10/#Variable_Weighting for details.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum AlternateHandling {
    /// altNonIgnorable turns off special handling of variables.
    #[default]
    NonIgnorable,

    /// altBlanked sets variables and all subsequent primary ignorables to be
    /// ignorable at all levels. This is identical to removing all variables
    /// and subsequent primary ignorables from the input.
    Blanked,

    /// altShifted sets variables to be ignorable for levels one through three and
    /// adds a fourth level based on the values of the ignored levels.
    Shifted,

    /// altShiftTrimmed is a slight variant of altShifted that is used to
    /// emulate POSIX.
    ShiftTrimmed,
}

// Go: collate/option.go:37 Option
/// An Option is used to change the behavior of a Collator. Options override the
/// settings passed through the locale identifier.
/// PORT: Go `collate.Option`; renamed so it does not shadow Rust `Option`.
#[derive(Clone, Copy)]
pub struct CollateOption {
    priority: i32,
    f: fn(&mut Options),
}

// Go: collate/option.go:56 options
pub struct Options {
    /// ignore specifies which levels to ignore.
    ignore: [bool; NUM_LEVELS],

    /// caseLevel is true if there is an additional level of case matching
    /// between the secondary and tertiary levels.
    case_level: bool,

    /// backwards specifies the order of sorting at the secondary level.
    /// This option exists predominantly to support reverse sorting of accents in French.
    backwards: bool,

    /// numeric specifies whether any sequence of decimal digits (category is Nd)
    /// is sorted at a primary level with its numeric value.
    /// For example, "A-21" < "A-123".
    /// This option is set by wrapping the main Weighter with NewNumericWeighter.
    numeric: bool,

    /// alternate specifies an alternative handling of variables.
    alternate: AlternateHandling,

    /// variableTop is the largest primary value that is considered to be
    /// variable.
    #[allow(dead_code)]
    variable_top: u32,

    t: Rc<dyn Weighter>,
    // PORT: Go `f norm.Form` (always `norm.NFD`) is not read by
    // `CompareString`; it is not ported.
}

// Go: collate/option.go:16 newCollator
/// newCollator creates a new collator with default options configured.
fn new_collator(t: Rc<dyn Weighter>) -> Collator {
    // Initialize a collator with default options.
    let mut ignore = [false; NUM_LEVELS];
    ignore[QUATERNARY] = true;
    ignore[IDENTITY] = true;
    // TODO: store vt in tags or remove.
    let variable_top = t.top();
    Collator {
        options: Options {
            ignore,
            case_level: false,
            backwards: false,
            numeric: false,
            alternate: AlternateHandling::NonIgnorable,
            variable_top,
            t: t.clone(),
        },
        iter: [CollIter::new(t.clone()), CollIter::new(t)],
    }
}

impl Options {
    // Go: collate/option.go:86 (*options).setOptions

    // Go: collate/option.go:102 (*options).setFromTag
}

// Go: collate/option.go:143 ldmlBool

// Go: collate/option.go:154 (option values)






// Go: collate/option.go:181 ignoreWidthF
fn ignore_width_f(o: &mut Options) {
    o.ignore[TERTIARY] = true;
    o.case_level = true;
}

// Go: collate/option.go:186 ignoreDiacriticsF
fn ignore_diacritics_f(o: &mut Options) {
    o.ignore[SECONDARY] = true;
}

// Go: collate/option.go:190 ignoreCaseF
fn ignore_case_f(o: &mut Options) {
    o.ignore[TERTIARY] = true;
    o.case_level = false;
}

// Go: collate/option.go:195 looseF
fn loose_f(o: &mut Options) {
    ignore_width_f(o);
    ignore_diacritics_f(o);
    ignore_case_f(o);
}

// Go: collate/option.go:201 forceF
fn force_f(o: &mut Options) {
    o.ignore[IDENTITY] = false;
}

// Go: collate/option.go:205 numericF
fn numeric_f(o: &mut Options) {
    o.numeric = true;
}

// ---------------------------------------------------------------------------
// collate/collate.go
// ---------------------------------------------------------------------------

// Go: collate/collate.go:53 tags

// Go: collate/collate.go:23 Collator
/// Collator provides functionality for comparing strings for a given
/// collation order.
/// PORT: Go `sorter` is only used by `Sort` and `SortStrings`, which are not
/// ported.
pub struct Collator {
    options: Options,

    iter: [CollIter; 2],
}

// Go: collate/collate.go:56 New

impl Collator {
    // Go: collate/collate.go:78 (*Collator).init

    // Go: collate/collate.go:121 (*Collator).CompareString

    // Go: collate/collate.go:157 (*Collator).compare
}

// Go: collate/collate.go:139 compareLevel
fn compare_level(f: fn(&mut CollIter) -> i32, a: &mut CollIter, b: &mut CollIter) -> i32 {
    a.pce = 0;
    b.pce = 0;
    loop {
        let va = f(a);
        let vb = f(b);
        if va != vb {
            if va < vb {
                return -1;
            }
            return 1;
        } else if va == 0 {
            break;
        }
    }
    0
}

// Go: collate/collate.go:234 iter
/// PORT: Go embeds `colltab.Iter`; here it is the field `it`.
struct CollIter {
    it: Iter,
    pce: usize,
}

impl CollIter {
    fn new(t: Rc<dyn Weighter>) -> CollIter {
        CollIter {
            it: Iter::new(t),
            pce: 0,
        }
    }

    // Go: collate/collate.go:241 (*iter).init
    fn init(&mut self, t: Rc<dyn Weighter>) {
        self.it.weighter = t;
        self.it.elems.clear();
    }

    // Go: collate/collate.go:246 (*iter).nextPrimary

    // Go: collate/collate.go:260 (*iter).nextSecondary
    fn next_secondary(&mut self) -> i32 {
        while self.pce < self.it.elems.len() {
            let v = self.it.elems[self.pce].secondary();
            if v != 0 {
                self.pce += 1;
                return v;
            }
            self.pce += 1;
        }
        0
    }

    // Go: collate/collate.go:270 (*iter).prevSecondary
    fn prev_secondary(&mut self) -> i32 {
        while self.pce < self.it.elems.len() {
            let v = self.it.elems[self.it.elems.len() - self.pce - 1].secondary();
            if v != 0 {
                self.pce += 1;
                return v;
            }
            self.pce += 1;
        }
        0
    }

    // Go: collate/collate.go:280 (*iter).nextTertiary
    fn next_tertiary(&mut self) -> i32 {
        while self.pce < self.it.elems.len() {
            let v = self.it.elems[self.pce].tertiary();
            if v != 0 {
                self.pce += 1;
                return i32::from(v);
            }
            self.pce += 1;
        }
        0
    }

    // Go: collate/collate.go:290 (*iter).nextQuaternary
    fn next_quaternary(&mut self) -> i32 {
        while self.pce < self.it.elems.len() {
            let v = self.it.elems[self.pce].quaternary();
            if v != 0 {
                self.pce += 1;
                return v;
            }
            self.pce += 1;
        }
        0
    }
}

// ---------------------------------------------------------------------------
// Go standard library and golang.org/x/text/unicode/norm helpers
// ---------------------------------------------------------------------------

/// Go `utf8.RuneSelf`.
const UTF8_RUNE_SELF: u8 = 0x80;

/// Go `utf8.RuneStart`.
fn utf8_rune_start(b: u8) -> bool {
    b & 0xC0 != 0x80
}


/// Go `unicode.In(r, unicode.Nd)` (go1.27.1, Unicode 17.0.0).
fn unicode_is_nd(r: u32) -> bool {
    unicode_is(&unicode_tables::ND, r)
}

// Go: unicode/letter.go:163 Is
/// Is reports whether the rune is in the specified table of ranges.
/// PORT: Go's linear and binary searches (`is16`, `is32`) give the same
/// answer as this scan.
fn unicode_is(range_tab: &RangeTable, r: u32) -> bool {
    let r16 = range_tab.r16;
    if let Some(&(_, hi, _)) = r16.last()
        && r <= u32::from(hi)
    {
        return r16.iter().any(|&(lo, hi, stride)| {
            let (lo, hi, stride) = (u32::from(lo), u32::from(hi), u32::from(stride));
            lo <= r && r <= hi && (stride == 1 || (r - lo) % stride == 0)
        });
    }
    let r32 = range_tab.r32;
    if let Some(&(lo, _, _)) = r32.first()
        && r >= lo
    {
        return r32.iter().any(|&(lo, hi, stride)| {
            lo <= r && r <= hi && (stride == 1 || (r - lo) % stride == 0)
        });
    }
    false
}

// ---------------------------------------------------------------------------
// Data
// ---------------------------------------------------------------------------

// Go: golang.org/x/text v0.38.0 collate/tables.go (CLDR 23). The big tables
// are the binary dumps in data/ (see the module note). Do not edit by hand.

/// Go `availableLocales`.
const AVAILABLE_LOCALES: &str = "und,aa,af,ar,as,az,be,bg,bn,bs,bs-Cyrl,ca,cs,cy,da,de-u-co-phonebk,de,dz,ee,el,en,en-US,en-US-u-va-posix,eo,es,et,fa,fa-AF,fi,fi-u-co-standard,fil,fo,fr,fr-CA,gu,ha,haw,he,hi,hr,hu,hy,ig,is,ja,kk,kl,km,kn,ko,kok,ln-u-co-phonetic,ln,lt,lv,mk,ml,mr,mt,my,nb,nn,nso,om,or,pa,pl,ps,ro,ru,se,si,sk,sl,sq,sr,sr-Latn,ssy,sv,sv-u-co-standard,ta,te,th,tn,to,tr,uk,ur,vi,wae,yo,zh,zh-u-co-stroke,zh-Hant-u-co-pinyin,zh-Hant";

/// Go `varTop`.
const VAR_TOP: u32 = 0x30e;

/// Go `locales`: (lookupOffset, valuesOffset) per available locale.
static LOCALES: [TableIndex; 95] = [
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x0,
    }, // und
    TableIndex {
        lookup_offset: 0x1c,
        values_offset: 0x1b4,
    }, // aa
    TableIndex {
        lookup_offset: 0x1d,
        values_offset: 0x0,
    }, // af
    TableIndex {
        lookup_offset: 0x1f,
        values_offset: 0x0,
    }, // ar
    TableIndex {
        lookup_offset: 0x21,
        values_offset: 0x0,
    }, // as
    TableIndex {
        lookup_offset: 0x27,
        values_offset: 0x1d7,
    }, // az
    TableIndex {
        lookup_offset: 0x28,
        values_offset: 0x0,
    }, // be
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x0,
    }, // bg
    TableIndex {
        lookup_offset: 0x2a,
        values_offset: 0x0,
    }, // bn
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x0,
    }, // bs
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x0,
    }, // bs-Cyrl
    TableIndex {
        lookup_offset: 0x2b,
        values_offset: 0x1ec,
    }, // ca
    TableIndex {
        lookup_offset: 0x2d,
        values_offset: 0x1f0,
    }, // cs
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x1f5,
    }, // cy
    TableIndex {
        lookup_offset: 0x30,
        values_offset: 0x1f7,
    }, // da
    TableIndex {
        lookup_offset: 0x32,
        values_offset: 0x201,
    }, // de-u-co-phonebk
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x0,
    }, // de
    TableIndex {
        lookup_offset: 0x34,
        values_offset: 0x0,
    }, // dz
    TableIndex {
        lookup_offset: 0x3a,
        values_offset: 0x20a,
    }, // ee
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x0,
    }, // el
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x0,
    }, // en
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x0,
    }, // en-US
    TableIndex {
        lookup_offset: 0x41,
        values_offset: 0x219,
    }, // en-US-u-va-posix
    TableIndex {
        lookup_offset: 0x42,
        values_offset: 0x23b,
    }, // eo
    TableIndex {
        lookup_offset: 0x43,
        values_offset: 0x23f,
    }, // es
    TableIndex {
        lookup_offset: 0x49,
        values_offset: 0x242,
    }, // et
    TableIndex {
        lookup_offset: 0x4b,
        values_offset: 0x0,
    }, // fa
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x0,
    }, // fa-AF
    TableIndex {
        lookup_offset: 0x4e,
        values_offset: 0x25a,
    }, // fi
    TableIndex {
        lookup_offset: 0x54,
        values_offset: 0x265,
    }, // fi-u-co-standard
    TableIndex {
        lookup_offset: 0x43,
        values_offset: 0x272,
    }, // fil
    TableIndex {
        lookup_offset: 0x30,
        values_offset: 0x1f7,
    }, // fo
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x0,
    }, // fr
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x0,
    }, // fr-CA
    TableIndex {
        lookup_offset: 0x56,
        values_offset: 0x0,
    }, // gu
    TableIndex {
        lookup_offset: 0x57,
        values_offset: 0x275,
    }, // ha
    TableIndex {
        lookup_offset: 0x5e,
        values_offset: 0x27a,
    }, // haw
    TableIndex {
        lookup_offset: 0x5f,
        values_offset: 0x0,
    }, // he
    TableIndex {
        lookup_offset: 0x61,
        values_offset: 0x0,
    }, // hi
    TableIndex {
        lookup_offset: 0x63,
        values_offset: 0x291,
    }, // hr
    TableIndex {
        lookup_offset: 0x65,
        values_offset: 0x297,
    }, // hu
    TableIndex {
        lookup_offset: 0x66,
        values_offset: 0x0,
    }, // hy
    TableIndex {
        lookup_offset: 0x68,
        values_offset: 0x29f,
    }, // ig
    TableIndex {
        lookup_offset: 0x6a,
        values_offset: 0x2a3,
    }, // is
    TableIndex {
        lookup_offset: 0x76,
        values_offset: 0x0,
    }, // ja
    TableIndex {
        lookup_offset: 0x77,
        values_offset: 0x0,
    }, // kk
    TableIndex {
        lookup_offset: 0x78,
        values_offset: 0x414,
    }, // kl
    TableIndex {
        lookup_offset: 0x7a,
        values_offset: 0x0,
    }, // km
    TableIndex {
        lookup_offset: 0x7c,
        values_offset: 0x0,
    }, // kn
    TableIndex {
        lookup_offset: 0x88,
        values_offset: 0x0,
    }, // ko
    TableIndex {
        lookup_offset: 0x8a,
        values_offset: 0x0,
    }, // kok
    TableIndex {
        lookup_offset: 0x8b,
        values_offset: 0x570,
    }, // ln-u-co-phonetic
    TableIndex {
        lookup_offset: 0x8b,
        values_offset: 0x0,
    }, // ln
    TableIndex {
        lookup_offset: 0x91,
        values_offset: 0x574,
    }, // lt
    TableIndex {
        lookup_offset: 0x93,
        values_offset: 0x582,
    }, // lv
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x0,
    }, // mk
    TableIndex {
        lookup_offset: 0x95,
        values_offset: 0x0,
    }, // ml
    TableIndex {
        lookup_offset: 0x97,
        values_offset: 0x0,
    }, // mr
    TableIndex {
        lookup_offset: 0x9a,
        values_offset: 0x58a,
    }, // mt
    TableIndex {
        lookup_offset: 0x9c,
        values_offset: 0x0,
    }, // my
    TableIndex {
        lookup_offset: 0x30,
        values_offset: 0x593,
    }, // nb
    TableIndex {
        lookup_offset: 0x30,
        values_offset: 0x593,
    }, // nn
    TableIndex {
        lookup_offset: 0x9e,
        values_offset: 0x595,
    }, // nso
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x59b,
    }, // om
    TableIndex {
        lookup_offset: 0xa0,
        values_offset: 0x0,
    }, // or
    TableIndex {
        lookup_offset: 0xa2,
        values_offset: 0x0,
    }, // pa
    TableIndex {
        lookup_offset: 0xa4,
        values_offset: 0x5a1,
    }, // pl
    TableIndex {
        lookup_offset: 0xa7,
        values_offset: 0x0,
    }, // ps
    TableIndex {
        lookup_offset: 0xa9,
        values_offset: 0x5b3,
    }, // ro
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x0,
    }, // ru
    TableIndex {
        lookup_offset: 0xab,
        values_offset: 0x5ba,
    }, // se
    TableIndex {
        lookup_offset: 0xad,
        values_offset: 0x0,
    }, // si
    TableIndex {
        lookup_offset: 0xaf,
        values_offset: 0x5c7,
    }, // sk
    TableIndex {
        lookup_offset: 0xb0,
        values_offset: 0x5cc,
    }, // sl
    TableIndex {
        lookup_offset: 0xb2,
        values_offset: 0x5cf,
    }, // sq
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x0,
    }, // sr
    TableIndex {
        lookup_offset: 0x15,
        values_offset: 0x0,
    }, // sr-Latn
    TableIndex {
        lookup_offset: 0x1c,
        values_offset: 0x1b4,
    }, // ssy
    TableIndex {
        lookup_offset: 0xb4,
        values_offset: 0x5d3,
    }, // sv
    TableIndex {
        lookup_offset: 0xb6,
        values_offset: 0x5d9,
    }, // sv-u-co-standard
    TableIndex {
        lookup_offset: 0xb8,
        values_offset: 0x0,
    }, // ta
    TableIndex {
        lookup_offset: 0xba,
        values_offset: 0x0,
    }, // te
    TableIndex {
        lookup_offset: 0xbc,
        values_offset: 0x0,
    }, // th
    TableIndex {
        lookup_offset: 0x9e,
        values_offset: 0x595,
    }, // tn
    TableIndex {
        lookup_offset: 0xbe,
        values_offset: 0x5e1,
    }, // to
    TableIndex {
        lookup_offset: 0xc4,
        values_offset: 0x5ed,
    }, // tr
    TableIndex {
        lookup_offset: 0xc5,
        values_offset: 0x0,
    }, // uk
    TableIndex {
        lookup_offset: 0xc7,
        values_offset: 0x0,
    }, // ur
    TableIndex {
        lookup_offset: 0xc9,
        values_offset: 0x5fc,
    }, // vi
    TableIndex {
        lookup_offset: 0xca,
        values_offset: 0x610,
    }, // wae
    TableIndex {
        lookup_offset: 0xcc,
        values_offset: 0x613,
    }, // yo
    TableIndex {
        lookup_offset: 0xe6,
        values_offset: 0x618,
    }, // zh
    TableIndex {
        lookup_offset: 0xff,
        values_offset: 0x618,
    }, // zh-u-co-stroke
    TableIndex {
        lookup_offset: 0xe6,
        values_offset: 0x618,
    }, // zh-Hant-u-co-pinyin
    TableIndex {
        lookup_offset: 0xff,
        values_offset: 0x618,
    }, // zh-Hant
];

// Go: collate/tables.go:399 mainExpandElem ([46864]uint32)

// Go: collate/tables.go:9191 mainContractElem ([4120]uint32)

// Go: collate/tables.go:9969 mainValues ([251456]uint32)

// Go: collate/tables.go:69446 mainLookup ([16576]uint16)

// Go: collate/tables.go:71257 mainCTEntries ([2529]struct{ L, H, N, I uint8 })
