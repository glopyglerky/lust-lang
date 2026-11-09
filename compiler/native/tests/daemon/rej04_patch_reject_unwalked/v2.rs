// A body patch whose collect fails (a template error in g2) after a
// const fn edit: h bakes k() into an array type, so it was a read-set
// walk candidate that kept its passing mark for a walk that never ran.
// v2 edits k and breaks g2; v3 fixes only g2, and warm must reject h
// ([u8; 3] from [0; 4], E0308). v4 puts k back.
#![allow(unused)]
const fn k() -> usize { let b = 4; b }
fn h() -> usize { let _a: [u8; 3] = [0; k()]; 1 }
fn g2<T>(_t: T) -> u32 { let _e: u32 = "x"; 0 }
fn main() { println!("{} {}", h(), g2(1u8)); }
