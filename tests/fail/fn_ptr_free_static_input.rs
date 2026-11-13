// expect: L0733
// rustc: lifetime may not live long enough: a fn that needs `&'static u8`
// becomes `fn(&'a u8)` only if `'a: 'static`.
fn hs(x: &'static u8) -> u8 { *x }
fn g<'a>(x: &'a u8) -> u8 { let f: fn(&'a u8) -> u8 = hs; f(x) }
fn main() { println!("{}", g(&1)); }
