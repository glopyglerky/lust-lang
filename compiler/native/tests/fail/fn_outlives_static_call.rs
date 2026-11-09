// expect: L0306
// rustc E0597: `fn f<'a: 'static>` makes its caller's borrow `'static`.
fn f<'a: 'static>(x: &'a u8) -> &'static u8 { x }
fn main() { let v = 3u8; println!("{}", f(&v)); }
