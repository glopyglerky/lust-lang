// expect: L0202
// rustc E0106: a lifetime inside a fn pointer param is not an input
// lifetime for elision.
fn f<'a>(_g: fn(&'a u8)) -> &u8 { &3 }
fn main() { let _ = f; }
