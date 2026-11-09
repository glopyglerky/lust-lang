//! Driver for the JSON kernel (medium tier of the milestone 4 kernels).
//!
//! Sources, copied from ts-rust `crates/ts_goport/src` (the
//! `goport_util` crate) with their module paths mirrored:
//!
//! - `frontend/json.rs`: the JSON v2 token decoder and the
//!   `MarshalerTo` / `UnmarshalerFrom` core (the milestone 4 JSON kernel).
//! - `frontend/json_ext.rs`: the v2 default arshalers the core calls for
//!   error texts, plus `LspAny` (a generic decoded value) and `AnyValue`.
//! - `frontend/json_indexmap.rs`: `IndexMap` decode (ordered map rules).
//! - `gostring.rs` as `crate::scanner_util`: the Go string port form.
//! - `gostd/strconv.rs`, `gostd/errors.rs`: quoting and Go errors.
//!
//! External crates are replaced in `support.rs`: `rustc-hash` (Fx maps),
//! `indexmap` (ordered map) and `memchr`. Edits inside copied files are
//! marked `KERNEL-EDIT`.
//!
//! The driver tokenizes and round-trips each input file given on the
//! command line, then runs fixed semantic cases. With no arguments it
//! runs only the fixed cases. Output must be byte-equal between rustc
//! and lust.

#![allow(dead_code, unused_imports, unused_variables, unused_mut, non_snake_case)]

pub mod frontend;
pub mod gostd;
mod support;

#[path = "gostring.rs"]
pub mod scanner_util;

// The kernel sources name the external crates directly. The root
// re-exports bind those names to the local stand-ins in `support.rs`.
pub use support::{indexmap, memchr, rustc_hash};

/// Kernel mirror of the ts-rust crate prelude: only the names the kernel
/// files use through the glob.
pub mod prelude {
    pub use crate::indexmap::{IndexMap, IndexSet};
    pub use crate::rustc_hash::{FxHashMap, FxHashSet};
    pub use crate::scanner_util::*;
    pub use crate::unported;
    pub use std::cell::RefCell;
    pub use std::rc::Rc;
}

use frontend::json::*;
use frontend::json_ext::LspAny;
use indexmap::IndexMap;
use rustc_hash::FxHashMap;

/// Dump the token stream of one JSON text. Stops at the first error.
fn dump_tokens(data: &[u8]) {
    let mut dec = json_new_decoder(data);
    let mut n = 0;
    loop {
        n += 1;
        match dec.read_token() {
            Ok(tok) => println!("tok {n}: {} {:?}", tok.kind() as char, tok),
            Err(e) => {
                println!("tok {n}: error: {}", e.message);
                return;
            }
        }
        if n > 100_000 {
            println!("tok: truncated");
            return;
        }
    }
}

/// Decode into a generic value and re-encode it, compact and indented.
fn round_trip(data: &[u8]) {
    let mut value = LspAny::default();
    match json_unmarshal(data, &mut value, &[]) {
        Ok(()) => {
            println!("value: {:?}", value);
            match json_marshal(&value, &[]) {
                Ok(s) => println!("compact: {s}"),
                Err(e) => println!("compact error: {}", e.message),
            }
            match json_marshal_indent(&value, "", "  ") {
                Ok(s) => println!("indent:\n{s}"),
                Err(e) => println!("indent error: {}", e.message),
            }
        }
        Err(e) => println!("decode error: {}", e.message),
    }
}

/// One input document: token stream, then the value round trip.
fn run_input(name: &str, data: &[u8]) {
    println!("== {name} ({} bytes)", data.len());
    dump_tokens(data);
    round_trip(data);
}

/// Fixed semantic cases: map merge rules, duplicate names, kind errors,
/// ordered maps and the raw value type. No output depends on hash order:
/// hash map contents are printed sorted by key.
fn fixed_cases() {
    println!("== fixed cases");

    // FxHashMap decode merges into the existing map (v2 default rules).
    let mut map: FxHashMap<String, String> = FxHashMap::default();
    map.insert("keep".to_string(), "old".to_string());
    let r = json_unmarshal(br#"{"b":"2","a":"1"}"#, &mut map, &[]);
    println!("fxmap merge ok: {:?}", r.is_ok());
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    for k in keys {
        println!("fxmap {k}={}", map[k]);
    }

    // Duplicate object names: rejected by default, allowed on request.
    let dup = br#"{"a":1,"a":2}"#;
    let mut v = LspAny::default();
    match json_unmarshal(dup, &mut v, &[]) {
        Ok(()) => println!("dup default: ok {:?}", v),
        Err(e) => println!("dup default error: {}", e.message),
    }
    let mut v = LspAny::default();
    match json_unmarshal(dup, &mut v, &[json_allow_duplicate_names(true)]) {
        Ok(()) => println!("dup allowed: {:?}", v),
        Err(e) => println!("dup allowed error: {}", e.message),
    }

    // Ordered map decode and encode keep insertion order.
    let mut ordered: IndexMap<String, LspAny> = IndexMap::new();
    let r = json_unmarshal(br#"{"z":1,"m":[true,null],"a":{"n":"x"}}"#, &mut ordered, &[]);
    println!("ordered ok: {:?}", r.is_ok());
    match json_marshal(&ordered, &[]) {
        Ok(s) => println!("ordered compact: {s}"),
        Err(e) => println!("ordered compact error: {}", e.message),
    }

    // Scalar decodes and kind mismatches (v2 SemanticError texts).
    let mut s = String::new();
    println!("str ok: {:?}", json_unmarshal(br#""hi \n""#, &mut s, &[]).is_ok());
    println!("str value: {s:?}");
    let mut s = String::new();
    match json_unmarshal(b"42", &mut s, &[]) {
        Ok(()) => println!("str from number: ok"),
        Err(e) => println!("str from number error: {}", e.message),
    }
    let mut b = false;
    match json_unmarshal(b"\"true\"", &mut b, &[]) {
        Ok(()) => println!("bool from string: ok {b}"),
        Err(e) => println!("bool from string error: {}", e.message),
    }
    let mut f = 0.0f64;
    println!("f64 ok: {:?}", json_unmarshal(b"-1.5e3", &mut f, &[]).is_ok());
    println!("f64 value: {f}");
    let mut f = 0.0f64;
    match json_unmarshal(b"null", &mut f, &[]) {
        Ok(()) => println!("f64 from null: ok {f}"),
        Err(e) => println!("f64 from null error: {}", e.message),
    }

    // Syntax errors carry the Go jsontext texts.
    for bad in [&b"{"[..], b"[1,]", b"'x'", b"{\"a\" 1}", b"tru", b"01", b"\"bad\x01\""] {
        let mut v = LspAny::default();
        match json_unmarshal(bad, &mut v, &[]) {
            Ok(()) => println!("bad {:?}: ok", String::from_utf8_lossy(bad)),
            Err(e) => println!("bad {:?} error: {}", String::from_utf8_lossy(bad), e.message),
        }
    }

    // Deep but legal nesting, built and checked both ways.
    let mut deep = LspAny::Number(1.0);
    for _ in 0..40 {
        deep = LspAny::Array(vec![deep]);
    }
    match json_marshal(&deep, &[]) {
        Ok(s) => {
            println!("deep len: {}", s.len());
            let mut back = LspAny::default();
            let ok = json_unmarshal(s.as_bytes(), &mut back, &[]).is_ok();
            println!("deep round trip: {}", ok && back == deep);
        }
        Err(e) => println!("deep error: {}", e.message),
    }

    // Escapes and unicode through a string value.
    let mut v = LspAny::default();
    let ok = json_unmarshal("\"a\\u00e9\\ud834\\udd1e\\t\\\\\"".as_bytes(), &mut v, &[]).is_ok();
    println!("escapes ok: {ok}");
    if let LspAny::String(s) = &v {
        let bytes: Vec<String> = s.as_bytes().iter().map(|b| format!("{b:02x}")).collect();
        println!("escapes bytes: {}", bytes.join(" "));
    }
    match json_marshal(&v, &[]) {
        Ok(s) => println!("escapes compact: {s}"),
        Err(e) => println!("escapes compact error: {}", e.message),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        fixed_cases();
        return;
    }
    for path in &args {
        let data = std::fs::read(path).expect("read input file");
        let name = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.clone());
        run_input(&name, &data);
    }
}
