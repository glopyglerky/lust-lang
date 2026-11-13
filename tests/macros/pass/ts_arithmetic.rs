// macro: arithmetic
// source: crates/ts_goport/src/jsnum.rs:204
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
use std::ops;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Number(pub f64);

macro_rules! arithmetic {
    ($trait:ident, $method:ident, $operator:tt) => {
        impl ops::$trait for Number {
            type Output = Self;

            fn $method(self, other: Self) -> Self {
                Self(self.0 $operator other.0)
            }
        }
    };
}

arithmetic!(Add, add, +);
arithmetic!(Sub, sub, -);
arithmetic!(Mul, mul, *);
arithmetic!(Div, div, /);

fn main() {
    let a = Number(6.0);
    let b = Number(4.0);
    println!("{} {} {} {}", (a + b).0, (a - b).0, (a * b).0, (a / b).0);
}
