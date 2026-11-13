// expect: L0612
// rustc E0277: `u8` has no `Neg` impl, so the bound does not hold.
use std::ops::Neg;

fn f<T: Neg<Output = T>>(a: T) -> T {
    a.neg()
}

fn main() {
    println!("{}", f(5u8));
}
