// Operator, `Deref`, and `Index` methods by name with the trait in scope:
// by name, `as _`, a glob, core's path, a fn-body use, and a param bound.
use core::ops::Not;
use std::ops::*;
mod named {
    use std::ops::Add as _;
    pub fn f() -> i32 {
        3i32.add(4)
    }
}
fn bound<T: std::ops::Neg<Output = T>>(t: T) -> T {
    t.neg()
}
fn main() {
    let s = String::from("ab");
    let mut x = 5i32;
    x.add_assign(2);
    println!("{} {} {} {}", s.deref(), true.not(), x, named::f());
    {
        use std::ops::Shl;
        println!("{} {}", 1u32.shl(4), bound(7i64));
    }
}
