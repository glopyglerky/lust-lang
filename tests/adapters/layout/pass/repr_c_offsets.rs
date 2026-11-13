// ARCH 4.4: offset_of! only on repr(C) types in this corpus.
// repr(C) keeps declaration order with C padding. A nested
// default-repr enum field keeps its own enum layout; only the
// repr(C) offsets are printed for it, not offsets inside it.
#![allow(dead_code)]
use std::mem::{align_of, offset_of, size_of};

#[repr(C)]
struct C {
    a: u8,
    b: u64,
    c: u8,
}
enum Inner {
    A(u8, u32),
    B,
}
#[repr(C)]
struct Outer {
    tag: u8,
    inner: Inner,
    tail: u16,
}
#[repr(C, align(64))]
struct Hot {
    flags: u32,
    object_flags: u32,
    id: u32,
    symbol: u32,
    alias: u64,
    data: [u8; 8],
}
#[repr(C)]
struct Nested {
    head: C,
    after: u8,
}
fn main() {
    println!("C size={} align={} a={} b={} c={}", size_of::<C>(), align_of::<C>(), offset_of!(C, a), offset_of!(C, b), offset_of!(C, c));
    println!(
        "Outer size={} align={} tag={} inner={} tail={} inner_size={}",
        size_of::<Outer>(),
        align_of::<Outer>(),
        offset_of!(Outer, tag),
        offset_of!(Outer, inner),
        offset_of!(Outer, tail),
        size_of::<Inner>(),
    );
    println!(
        "Hot size={} align={} flags={} id={} alias={} data={}",
        size_of::<Hot>(),
        align_of::<Hot>(),
        offset_of!(Hot, flags),
        offset_of!(Hot, id),
        offset_of!(Hot, alias),
        offset_of!(Hot, data),
    );
    println!("Nested size={} head={} head_b={} after={}", size_of::<Nested>(), offset_of!(Nested, head), offset_of!(Nested, head.b), offset_of!(Nested, after));
}
