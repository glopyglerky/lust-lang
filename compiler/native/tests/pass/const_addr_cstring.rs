// LLVM puts a fully init constant whose only zero byte is its last one in
// a C-string section, so the linker does not merge it with an equal-bytes
// constant that has uninit bytes (.rodata.cst4/8/16/32). Equal C strings
// still share one address, and a constant with a zero byte elsewhere is in
// the cstN section with the uninit ones (fx-niche NOTE 08:21).
use std::ptr::addr_eq;

#[derive(Clone, Copy)]
#[repr(u16)]
enum F {
    V0,
}
#[derive(Clone, Copy)]
#[repr(C)]
struct S3 {
    a: u16,
    b: u8,
}
#[derive(Clone, Copy)]
#[repr(C)]
struct S7 {
    a: u32,
    b: u16,
    c: u8,
}

const A: Result<(F, u8), bool> = Err(true);
const C: Result<(F, u8), bool> = Err(false);
const P: S3 = S3 { a: 0x0201, b: 3 };
const Q: S3 = S3 { a: 0x0201, b: 0 };
const L: S7 = S7 { a: 0x04030201, b: 0x0605, c: 7 };
static SP: &S3 = &S3 { a: 0x0201, b: 3 };

fn main() {
    // ff ff 01 00 is a C string; A has an uninit last byte.
    println!("{}", addr_eq(&A, &0x0001ffffu32));
    // ff ff 00 00 is not: both are in .rodata.cst4.
    println!("{}", addr_eq(&C, &0x0000ffffu32));
    println!("{} {}", addr_eq(&P, &[1u8, 2, 3, 0]), addr_eq(&P, &0x00030201u32));
    println!("{} {}", addr_eq(&Q, &[1u8, 2, 0, 0]), addr_eq(&Q, &[0x0201u16, 0]));
    println!("{} {}", addr_eq(&L, &[1u8, 2, 3, 4, 5, 6, 7, 0]), addr_eq(&L, &[1u8, 2, 3, 4, 5, 6, 0, 0]));
    // Equal C strings are one constant.
    println!("{} {}", addr_eq(&[1u8, 2, 3, 0], &0x00030201u32), addr_eq(&0x00030201u32, &[0x0201u16, 3]));
    println!("{}", addr_eq(SP, &0x00030201u32));
}
