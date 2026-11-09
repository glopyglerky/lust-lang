// `-&x` and `(&x).neg()` run core's `impl Neg for &i32` (made by
// forward_ref_unop!, not #[track_caller]), so the overflow reports
// core's arith.rs.
// panic: attempt to negate with overflow
use std::ops::Neg;
fn main() {
    let x: i64 = bb(i64::MIN + 1);
    println!("{} {}", -&x, (&x).neg());
    let y: i16 = bb(i16::MIN);
    println!("{}", -&y);
}
#[inline(never)]
fn bb<T>(x: T) -> T {
    x
}
