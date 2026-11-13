// expect: L0308
// rustc E0596: an impl FnMut value is called through a mutable borrow (W-CHECK-G3).
fn mutf(s: String) -> impl FnMut() -> usize { let mut s = s; move || { s.push('!'); s.len() } }
fn main() { let f = mutf(String::from("a")); println!("{}", f()); }
