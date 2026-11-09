use std::cmp::{min, max as biggest};
use std::vec::*;
use std::string::String;

struct S {
    pub(crate) a: i64,
    pub b: i64,
}

fn main() {
    let s = S { a: 2, b: 3 };
    println!("{}", s.a + s.b);
}
