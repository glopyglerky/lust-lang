// The literal is i32 (the fallback), not the `Into<i64>` target, so
// `t + t` overflows and panics, as in rustc.
use std::ops::Add;
fn f<T: Into<i64> + Add<Output = T> + Copy>(t: T) -> i64 {
    (t + t).into()
}
fn main() {
    println!("{}", f(1000));
    println!("{}", f(2000000000));
}
