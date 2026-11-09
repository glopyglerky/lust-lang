// expect: L0735
// rustc E0308: a fn pointer that returns its argument does not meet
// `Fn(&u8) -> &'static u8`.
fn call<F: Fn(&u8) -> &'static u8>(f: F) -> u8 { let a = 1; *f(&a) }
fn id(x: &u8) -> &u8 { x }
fn main() { let p: fn(&u8) -> &u8 = id; println!("{}", call(p)); }
