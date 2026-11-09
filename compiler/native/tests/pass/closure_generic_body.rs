// Closures inside generic fns: captures follow the generic body.
use std::ops::Add;
fn repeat_sum<T: Copy + Add<Output = T>>(t: T, zero: T, n: usize) -> T {
    let rep = |k: usize| {
        let mut s = zero;
        let mut i = 0;
        while i < k {
            s = s + t;
            i += 1;
        }
        s
    };
    rep(n)
}
fn twice<T: Copy, F: Fn(T) -> T>(f: F, t: T) -> T {
    let g = |x: T| f(f(x));
    g(t)
}
fn main() {
    println!("{}", repeat_sum(7u8, 0, 3));
    println!("{}", repeat_sum(5i64, 1, 2));
    println!("{}", twice(|x: u32| x * 3, 2));
    println!("{}", twice(|x: i64| x - 1, 0));
}
