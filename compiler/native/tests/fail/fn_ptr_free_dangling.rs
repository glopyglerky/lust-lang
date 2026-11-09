// expect: L0306
// rustc E0597: `fn(&'a str)` takes the caller's `'a`, which outlives the
// body, so a borrow of a local does not fit (W-BORROW3).
fn g<'a>(f: fn(&'a str)) { let s = String::new(); f(&s); }
fn main() { g(|_| {}); }
