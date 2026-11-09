// expect: L0735
// rustc: implementation of `Fn` is not general enough: `fn(&'static u8)`
// does not take every lifetime `F: Fn(&u8)` passes.
fn call<F: Fn(&u8) -> u8>(f: F) -> u8 { let a = 1; f(&a) }
fn id(x: &'static u8) -> u8 { *x }
fn main() { let p: fn(&'static u8) -> u8 = id; println!("{}", call(p)); }
