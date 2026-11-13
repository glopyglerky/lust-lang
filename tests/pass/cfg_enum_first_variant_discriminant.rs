// A cfg on an enum's first variant with an operator discriminant is fine:
// it is a variant, not a block tail.
#![allow(unused)]
#[derive(Debug)]
enum E {
    #[cfg(all())]
    A = 1 + 2,
    #[cfg(any())]
    B = 4 - 1,
}
fn main() {
    println!("{:?} {}", E::A, E::A as i32);
}
