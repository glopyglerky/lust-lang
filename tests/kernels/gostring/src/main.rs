//! Driver for the gostring kernel (small tier of the scanner kernels).
//!
//! Source: `crates/ts_goport/src/gostring.rs` in ts-rust, copied verbatim
//! into `src/gostring.rs`. In ts-rust this file is the bottom half of
//! `scanner_util` (module `goport_util::scanner_util`): the Go string port
//! form (marker U+FDD0), surrogate handling and Go byte offsets that the
//! scanner and the JSON kernel both build on.
//!
//! The driver runs a fixed case table and prints every result. Output
//! must be byte-equal between rustc and lust. Hex dumps keep the output
//! valid UTF-8 and independent of terminal encodings.

#![allow(dead_code, unused_imports, unused_variables, unused_mut, non_snake_case)]

mod gostring;
mod support;

// The kernel sources name the external crates directly (`memchr::…`).
// The root re-exports bind those names to the local stand-ins.
pub use support::memchr;

use gostring::*;

/// Print a string as space-separated hex bytes.
fn hex(label: &str, s: &str) {
    let mut out = String::new();
    for b in s.as_bytes() {
        out.push_str(&format!("{b:02x} "));
    }
    println!("{label}: {}", out.trim_end());
}

/// Print the Go units of a port-form string, left to right.
fn units(label: &str, s: &str) {
    let mut i = 0;
    let mut line = String::new();
    while i < s.len() {
        let (unit, size) = go_unit_at(s, i);
        match unit {
            GoUnit::Char(ch) => line.push_str(&format!("U+{:04X} ", ch as u32)),
            GoUnit::Surrogate(u) => line.push_str(&format!("SURR({u:04X}) ")),
            GoUnit::InvalidByte(b) => line.push_str(&format!("BYTE({b:02X}) ")),
        }
        if size == 0 {
            break;
        }
        i += size;
    }
    println!("{label}: {}", line.trim_end());
}

fn case(name: &str, raw: Vec<u8>) {
    println!("== {name}");
    // Go string from arbitrary bytes (the port form), then every view of it.
    let s = go_string_from_bytes(raw.clone());
    hex("port-form", &s);
    units("units", &s);
    println!("marker: {}", contains_go_string_marker(&s));
    println!("go_len: {}", go_len(&s));
    let bytes = go_string_bytes(&s);
    println!("bytes-back: {}", bytes.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" "));
    println!("round-trip: {}", bytes.as_ref() == raw.as_slice());
    hex("valid-utf8", &go_to_valid_utf8(&s));
    hex("go_value", &go_value(&s));
    let runes: Vec<String> = go_runes(&s).iter().map(|c| format!("U+{:04X}", *c as u32)).collect();
    println!("runes: {}", runes.join(" "));
    // Byte offset mapping, port form to Go form and back, at each unit.
    let mut i = 0;
    while i <= s.len() {
        let go = go_byte_offset(&s, i as i32);
        let back = port_byte_offset(&s, go);
        println!("offset port={i} go={go} back={back} before={}", go_offset_before(&s, i as i32));
        if i == s.len() {
            break;
        }
        let (_, size) = go_unit_at(&s, i);
        i += if size == 0 { 1 } else { size };
    }
    // Slices in Go byte space.
    let n = go_len(&s);
    if n >= 2 {
        hex("slice[1..n-1]", &go_slice(&s, 1, n - 1));
        hex("cut[0..2]", &go_cut_slice(&s, 0, 2));
    }
    println!("prefix-self: {}", go_has_prefix(&s, &s.clone()));
    println!("suffix-empty: {}", go_has_suffix(&s, ""));
    let upper = go_map_runes(&s, |c| c.to_ascii_uppercase());
    hex("map-runes-upper", &upper);
    println!("ord-self: {:?}", compare_go_bytes(&s, &s.clone()));
    println!("ord-str: {:?}", compare_go_strings(&s, &s.clone()));
}

fn main() {
    // Plain ASCII and multi-byte UTF-8.
    case("ascii", b"hello, world".to_vec());
    case("utf8", "héllo → wörld 𝄞".as_bytes().to_vec());
    // A real U+FDD0 in the input (the marker char itself).
    case("marker", "a\u{FDD0}b".as_bytes().to_vec());
    // Invalid UTF-8 bytes, as Go strings can hold.
    case("invalid", vec![b'a', 0xFF, 0xFE, b'b', 0x80, b'c']);
    // UTF-16 surrogate pair bytes (CESU-8 style, as JS strings produce).
    case("surrogate-pair", vec![0xED, 0xA0, 0xB4, 0xED, 0xB4, 0x9E]); // U+1D11E as two surrogates
    // Lone high surrogate.
    case("lone-surrogate", vec![b'x', 0xED, 0xA0, 0x80, b'y']);
    case("empty", Vec::new());

    // Surrogate predicates and pair fusion.
    for ch in [0xD7FFu32, 0xD800, 0xDBFF, 0xDC00, 0xDFFF, 0xE000, 0x1D11E] {
        println!(
            "surr {ch:05X}: high={} low={} any={}",
            is_high_surrogate(ch),
            is_low_surrogate(ch),
            is_surrogate(ch)
        );
    }
    println!("pair: {:05X}", surrogate_pair_to_code_point(0xD834, 0xDD1E));

    // JS string rune encode/decode (UTF-16 code units in port form).
    for ch in [0x41u32, 0xE9, 0x20AC, 0x1D11E, 0xD800, 0xFFFD, 0x10FFFF] {
        let enc = encode_js_string_rune(ch);
        let (dec, size) = decode_js_string_rune(&enc);
        println!("jsrune {ch:05X}: enc-len={} dec={dec:05X} size={size}", enc.len());
        let mut pushed = String::new();
        push_js_string_rune(&mut pushed, ch);
        println!("jsrune-push-same: {}", pushed == enc);
    }

    // Go string from a Rust UTF-8 string (the other entry point).
    let s = go_string_from_utf8("plain \u{FDD0} text".to_string());
    hex("from-utf8", &s);
    units("from-utf8-units", &s);
    let pair = go_string_from_bytes(vec![0xED, 0xA0, 0xB4, 0xED, 0xB4, 0x9E]);
    let fused = fuse_surrogate_bytes(&pair);
    hex("fused", &fused);

    // Offset table over a mixed string.
    let mixed = go_string_from_bytes(vec![b'a', 0xFF, 0xC3, 0xA9, 0xED, 0xA0, 0x80, b'z']);
    let offsets = GoOffsets::new(&mixed);
    for pos in 0..=(mixed.len() as i32) {
        println!("goffsets port={pos} go={}", offsets.go_offset(pos));
    }
    println!("cmp-slices: {:?}", compare_go_bytes_slices(&["a".into(), "b".into()], &["a".into(), "c".into()]));
}
