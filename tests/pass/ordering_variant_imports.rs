// Ordering's variants by name, by glob, and by fn-local glob, in
// patterns and expressions.
use std::cmp::Ordering::{self, Equal, Greater, Less};
fn classify(a: i32, b: i32) -> &'static str {
    match a.cmp(&b) {
        Less => "lt",
        Equal => "eq",
        Greater => "gt",
    }
}
fn main() {
    let o: Ordering = Greater;
    println!("{} {} {} {:?}", classify(1, 2), classify(2, 2), classify(3, 2), o);
    let v: Vec<Ordering> = vec![Less, Equal];
    println!("{:?} {}", v, matches!(v[0], Less));
}
