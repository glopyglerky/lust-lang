// expect: L0301
// rustc: borrow of moved value, in a parameter pattern.
fn f(a @ (ref b, _): (String, i32)) {} fn main() { f((String::new(), 1)); }
