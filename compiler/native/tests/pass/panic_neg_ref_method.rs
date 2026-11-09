// `(&x).neg()` reports core's arith.rs like `-&x`; `x.neg()` by value
// stays at the call.
// panic: attempt to negate with overflow
use std::ops::Neg;
fn main() {
    let x: i32 = bb(i32::MIN);
    let r = &x;
    println!("{}", r.neg());
}
#[inline(never)]
fn bb<T>(x: T) -> T {
    x
}
