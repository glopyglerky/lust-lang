// `size_of_val(&x)` where `x`'s literal type is still open at the call:
// the answer waits for the body end, so a later constraint (`b: u64 =
// a`) or the i32 / f64 fallback decides it, as in rustc (W-STD-C).
use std::mem;
fn main() {
    let a = 1;
    let b = 2;
    let c = 2.5;
    println!("{} {} {}", mem::size_of_val(&a), mem::size_of_val(&b), mem::size_of_val(&c));
    let d: u64 = b;
    let e = (1, 2.0);
    println!("{} {} {}", mem::size_of_val(&e), mem::align_of_val(&e), d);
    println!("{} {}", mem::size_of_val(&[0u16; 3]), mem::size_of_val("abc"));
}
