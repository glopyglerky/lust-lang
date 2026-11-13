// Tuple field forms: `offset_of!(W, 1)` on a tuple struct, a tuple
// struct literal with index fields and `..base`, and a reference to
// an align-1 packed field (allowed, unlike wider packed fields).
use std::mem::{offset_of, size_of};

#[repr(C)]
struct W(u8, u16, u8);

#[derive(Debug)]
struct T(i32, i32);

#[repr(C, packed)]
struct P {
    a: u8,
    b: u32,
    c: u64,
}

fn main() {
    println!("{} {} {}", size_of::<W>(), offset_of!(W, 1), offset_of!(W, 2));

    let a = T(1, 2);
    let b = T { 0: 5, ..a };
    println!("{:?}", b);

    let mut p = P { a: 1, b: 0xdeadbeef, c: 77 };
    let r = &p.a;
    let bv = p.b;
    p.c += 1;
    let cv = p.c;
    println!("{} {} {} {}", r, bv, cv, size_of::<P>());
}
