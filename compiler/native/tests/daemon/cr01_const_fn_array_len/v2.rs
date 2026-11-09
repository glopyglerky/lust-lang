// A const fn body edit moves values bodies baked in through the scalar
// evaluator (an array repeat length, a length in a type): check/
// scalar_reads.rs stores them as Checked reads. No generic fn here, so the
// read-set walk alone decides. v2 makes k() 4, so both uses reject
// (E0308); v3 puts it back; v4 edits k2 and other() but keeps the value.
#![allow(unused)]
const fn k2() -> usize { 2 + 2 }
const fn k() -> usize { k2() }
fn h() -> usize { let _a: [u8; 3] = [0; k()]; 1 }
fn t() -> usize { let _b: [u16; k()] = [7; 3]; 2 }
fn other() -> u32 { 7 }
fn main() { println!("{} {} {}", h(), t(), other()); }
