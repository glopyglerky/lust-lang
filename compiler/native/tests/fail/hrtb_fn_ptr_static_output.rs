// expect: L0110
// rustc E0308: a fn that returns its argument does not become a
// `for<'a> fn(&'a u8) -> &'static u8`.
fn ap(f: for<'a> fn(&'a u8) -> &'static u8) -> u8 { *f(&1) }
fn id(x: &u8) -> &u8 { x }
fn main() { println!("{}", ap(id)); }
