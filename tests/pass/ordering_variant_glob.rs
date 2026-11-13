// A glob of Ordering's variants, at module level.
use std::cmp::Ordering::{self, *};
fn sign(n: i32) -> &'static str {
    match n.cmp(&0) {
        Less => "neg",
        Equal => "zero",
        Greater => "pos",
    }
}
fn main() {
    let o: Ordering = Less;
    println!("{} {} {} {:?}", sign(-1), sign(0), sign(5), o);
}
