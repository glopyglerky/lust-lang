// expect: L0612
// rustc E0277: `String` has several `PartialEq<Rhs>` impls, so
// `s.eq(&&s)` takes `Rhs` from the argument (`&String`) with no deref
// coercion, and `String: PartialEq<&String>` does not hold.
fn main() {
    let s = String::from("a");
    println!("{}", s.eq(&&s));
}
