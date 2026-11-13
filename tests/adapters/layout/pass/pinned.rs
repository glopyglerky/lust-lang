// ARCH 4.4 gate: every shape pinned by name in section 4.4,
// each with its rustc size as a const assert and as printed output.
#![allow(dead_code)]
use std::mem::{align_of, size_of};

enum E1 {
    A(u8, u32),
    B,
}
enum E1b {
    A(u8, u64, u8),
    B,
}
enum E1c {
    A(u32, u64),
    B,
}
enum E1d {
    A(u16, [u8; 3]),
    B,
}
// TypeMapper-like: an Array(Box<_>, bool) variant beside others.
enum MapperLike<B> {
    Leaf(u32),
    Array(Box<B>, bool),
    Map(Box<B>),
    None,
}
struct S {
    a: bool,
    b: &'static u8,
}
struct Payload {
    pointer: &'static u8,
    flag: bool,
}
enum Several {
    Data(Payload),
    Empty1,
    Empty2,
}
enum OtherData {
    Pointer(&'static u8),
    Byte(u8),
}
#[repr(u8)]
enum ExplicitTag {
    Pointer(&'static u8),
    Empty,
}
#[repr(C)]
enum CTag {
    Pointer(&'static u8),
    Empty,
}
enum E3 {
    A(u32),
    B(u32),
}

const _: () = assert!(size_of::<E1>() == 8);
const _: () = assert!(size_of::<E1b>() == 16);
const _: () = assert!(size_of::<E1c>() == 16);
const _: () = assert!(size_of::<E1d>() == 6);
const _: () = assert!(size_of::<MapperLike<u8>>() == 16);
const _: () = assert!(size_of::<Option<Option<S>>>() == 16);
const _: () = assert!(size_of::<Several>() == 16);
const _: () = assert!(size_of::<OtherData>() == 16);
const _: () = assert!(size_of::<ExplicitTag>() == 16);
const _: () = assert!(size_of::<CTag>() == 16);
// Tag widening: E3 gets a 4-byte tag, so three nested Options stay.
const _: () = assert!(size_of::<Option<Option<Option<E3>>>>() == 8);
// Plain struct reorder: declaration order would give 24.
struct SMixed {
    a: u8,
    b: u64,
    c: u8,
}
const _: () = assert!(size_of::<SMixed>() == 16);
const _: () = assert!(size_of::<Option<bool>>() == 1);
enum Disc {
    A = 1000,
    B,
}
const _: () = assert!(size_of::<Disc>() == 2);

fn p<T>(name: &str) {
    println!("{name} size={} align={}", size_of::<T>(), align_of::<T>());
}
fn main() {
    p::<E1>("E1_A_u8_u32_B");
    p::<E1b>("E1b_A_u8_u64_u8");
    p::<E1c>("E1c_A_u32_u64");
    p::<E1d>("E1d_A_u16_arr3");
    p::<MapperLike<u8>>("MapperLike");
    p::<S>("S_bool_ref");
    p::<Option<S>>("OptS");
    p::<Option<Option<S>>>("OptOptS");
    p::<Payload>("Payload");
    p::<Several>("Several");
    p::<OtherData>("OtherData");
    p::<ExplicitTag>("ExplicitTag_repr_u8");
    p::<CTag>("CTag_repr_C");
    p::<E3>("E3_A_u32_B_u32");
    p::<Option<E3>>("OptE3");
    p::<Option<Option<Option<E3>>>>("Opt3E3");
    p::<SMixed>("SMixed_u8_u64_u8");
    p::<Option<bool>>("OptBool");
    p::<Disc>("Disc_1000");
    println!("Disc_A {}", Disc::A as i32);
    println!("Disc_B {}", Disc::B as i32);
}
