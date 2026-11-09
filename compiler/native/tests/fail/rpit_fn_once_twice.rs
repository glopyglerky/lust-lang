// expect: L0301
// rustc E0382: an impl FnOnce value is consumed by its call (W-CHECK-G3).
fn once(s: String) -> impl FnOnce() -> String { move || s }
fn main() { let f = once(String::from("a")); println!("{}", f()); println!("{}", f()); }
