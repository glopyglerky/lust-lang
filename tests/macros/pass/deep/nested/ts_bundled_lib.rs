// macro: bundled_lib
// source: crates/ts_goport/src/frontend/bundled/embed.rs:358
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
// The wasm def below is cfg-stripped on this host target. The static
// and main use the not-wasm tuple shape, as ts-rust host builds do.
// This file sits three directories below tests/macros/ on purpose: the
// macro includes concat!("../../../libs/", $name) relative to the
// invoking file, which resolves to tests/macros/libs/ from here.
pub const fn snapshot_text_hash(text: &str) -> u64 {
    let b = text.as_bytes();
    let mut h: u64 = 0xcbf29ce484222325;
    let mut i = 0;
    while i < b.len() { h = (h ^ b[i] as u64).wrapping_mul(0x100000001b3); i += 1; }
    h
}
pub type EmbeddedLib = (&'static str, &'static str, u64);

#[cfg(not(target_family = "wasm"))]
macro_rules! bundled_lib {
    ($name:literal) => {{
        const TEXT: &str = include_str!(concat!("../../../libs/", $name));
        // One compile-time evaluation per lib, so each stays under the
        // rustc step limit.
        const HASH: u64 = snapshot_text_hash(TEXT);
        (concat!("libs/", $name), TEXT, HASH)
    }};
}

#[cfg(target_family = "wasm")]
macro_rules! bundled_lib {
    ($name:literal) => {
        concat!("libs/", $name)
    };
}

static EMBEDDED: &[EmbeddedLib] = &[bundled_lib!("sample.d.ts")];

fn main() {
    let (path, text, hash) = EMBEDDED[0];
    println!("path={path} text='{}' hash_nonzero={}", text.trim(), hash != 0);
}
