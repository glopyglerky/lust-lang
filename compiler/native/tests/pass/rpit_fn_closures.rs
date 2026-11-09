// impl FnOnce and impl FnMut return values called through their bounds (W-CHECK-G3).
fn once(s: String) -> impl FnOnce() -> String { move || s }
fn mutf(s: String) -> impl FnMut() -> usize { let mut s = s; move || { s.push('!'); s.len() } }
fn main() { let f = once(String::from("a")); println!("{}", f()); let mut g = mutf(String::from("b")); println!("{} {}", g(), g()); }
