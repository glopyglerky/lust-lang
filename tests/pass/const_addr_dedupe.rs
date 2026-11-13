// Equal constants share one address, as rustc gives them in one codegen
// unit: promoted temps, literals, const pointees and static pointees, also
// across types (equal bytes, uninit runs and pointer places; a 4, 8, 16 or
// 32 byte constant with no pointer by its bytes alone). A static itself is
// never shared. Static pointees of each key shape (S8 to SL) check that the
// hash codegen computes for a pointee (cgen/kconst.rs khash) is the one the
// run-time table computes for an equal literal (main interns at run time).
// Every comparison is inside one fn: between fns rustc's answer depends on
// its codegen unit split (docs: rule zero).
use std::ptr;

#[derive(Clone, Copy)]
struct P {
    a: u8,
    b: u8,
}
#[derive(Clone, Copy)]
enum E {
    A,
    B(u8),
    C(u32),
}

const C: &i32 = &1;
const D: &[u8] = b"xy";
static SP: &i32 = &1;
static SS: &str = "ab";
static SB: &[u8; 2] = b"ab";
static ST: i32 = 1;
static S1: &[u8] = &[1, 0, 0, 0];
static S2: &[u8; 4] = b"\x01\x00\x00\x00";
static S8: &[u8; 8] = &[1, 2, 3, 4, 5, 6, 7, 8];
static S16: &[u32; 4] = &[1, 2, 3, 4];
static S32: &[u16; 16] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
static SZ: &[u8; 4] = b"abc\0";
static SL: &str = "a static str of more than 16 bytes";

fn main() {
    println!("ints {} {} {}", ptr::eq(&1, &1), ptr::eq(&1u8, &1u8), ptr::eq(&1, &2));
    println!("types {} {} {}", ptr::addr_eq(&1i32, &1u32), ptr::addr_eq(&1u8, &true), ptr::addr_eq(&0u32, &0.0f32));
    println!("sizes {} {}", ptr::addr_eq(&1u8, &1u16), ptr::addr_eq(&0x01020304i32, &[4u8, 3, 2, 1]));
    println!("str {} {}", "ab".as_ptr() == "ab".as_ptr(), "ab".as_ptr() == "abc".as_ptr());
    println!("bytes {} {}", "ab".as_ptr() == b"ab".as_ptr(), "ab".as_ptr() == (&[b'a', b'b']).as_ptr());
    println!("arrays {} {}", ptr::eq(&[1u64, 2, 3, 4, 5], &[1u64, 2, 3, 4, 5]), [1u8, 2][..].as_ptr() == (&[1u8, 2]).as_ptr());
    println!("struct {} {}", ptr::addr_eq(&P { a: 1, b: 0 }, &1u16), ptr::eq(&E::B(7), &E::B(7)));
    println!("uninit {} {}", ptr::addr_eq(&None::<u32>, &0u64), ptr::addr_eq(&E::A, &E::C(0)));
    println!("padding {}", ptr::addr_eq(&(1u8, 2u16), &0x0001_0002u32));
    println!("nested {} {}", ptr::eq(&&1, &&1), ptr::eq(*&&1, &1));
    println!("const {} {} {}", ptr::eq(C, &1), ptr::eq(C, SP), D.as_ptr() == b"xy".as_ptr());
    println!("static {} {} {}", SS.as_ptr() == SB.as_ptr(), SS.as_ptr() == "ab".as_ptr(), ptr::eq(&ST, &1));
    println!("statics {} {}", S1.as_ptr() == S2.as_ptr(), ptr::addr_eq(S1.as_ptr(), &1u32));
    println!("words {} {} {}", ptr::addr_eq(S8, &0x0807060504030201u64), ptr::addr_eq(S16, &[0x0000_0002_0000_0001u64, 0x0000_0004_0000_0003]), ptr::eq(S32, &[1u16, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]));
    println!("long {} {}", SZ.as_ptr() == b"abc\0".as_ptr(), SL.as_ptr() == "a static str of more than 16 bytes".as_ptr());
    println!("casts {} {}", &7 as *const i32 as usize == &7 as *const i32 as usize, format!("{:p}", &9) == format!("{:p}", &9));
    let mut v = Vec::new();
    for _ in 0..3 {
        v.push(&3 as *const i32);
    }
    println!("loop {}", v[0] == v[1] && v[1] == v[2]);
}
