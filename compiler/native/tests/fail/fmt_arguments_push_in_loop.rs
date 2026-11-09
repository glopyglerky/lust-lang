// expect: L0306
// The arguments array of a `format_args!` value is a temporary of its
// statement: the value cannot be kept past it (rustc E0716).
use std::fmt;

fn main() {
    let arr = [1, 2, 3];
    let mut v: Vec<fmt::Arguments> = Vec::new();
    for r in &arr {
        v.push(format_args!("{}", *r));
    }
    println!("{}", v.len());
}
