//! Port of Effect-TS/tsgo `internal/keybuilder`: the deterministic keys of
//! service, tag and error declarations.

use crate::effect::etscore::KeyPattern;
use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: keybuilder/keybuilder.go Cyrb53
/// Cyrb53 computes a fast non-cryptographic hash of the input string,
/// producing a 16-character zero-padded hex string from two uint32 halves.
/// This is a Go port of the cyrb53 function from the reference TS implementation.
pub fn cyrb53(str: &str) -> String {
    let mut h1: u32 = 0xdead_beef;
    let mut h2: u32 = 0x41c6_ce57;

    for &b in str.as_bytes() {
        let ch = u32::from(b);
        h1 = imul(h1 ^ ch, 2_654_435_761);
        h2 = imul(h2 ^ ch, 1_597_334_677);
    }

    h1 = imul(h1 ^ (h1 >> 16), 2_246_822_507);
    h1 ^= imul(h2 ^ (h2 >> 13), 3_266_489_909);
    h2 = imul(h2 ^ (h2 >> 16), 2_246_822_507);
    h2 ^= imul(h1 ^ (h1 >> 13), 3_266_489_909);

    format!("{h2:08x}{h1:08x}")
}

// Go: keybuilder/keybuilder.go imul
/// imul emulates JavaScript's Math.imul: 32-bit integer multiplication
/// that discards overflow (keeps only the lower 32 bits).
fn imul(a: u32, b: u32) -> u32 {
    a.wrapping_mul(b)
}

/// Go `strings.ToLower`: `unicode.ToLower` on each rune.
fn strings_to_lower(s: &str) -> String {
    s.chars().map(crate::gostd::unicode::to_lower).collect()
}

// Go: keybuilder/keybuilder.go CreateString
