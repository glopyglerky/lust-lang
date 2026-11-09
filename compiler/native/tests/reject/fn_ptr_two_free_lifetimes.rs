// expect: L0203
// Valid Rust: a fn pointer type that names two lifetimes of the fn. lust
// keeps one free region per fn pointer type (W-BORROW3).
fn g<'a, 'b>(f: fn(&'a u8, &'b u8) -> u8, x: &'a u8, y: &'b u8) -> u8 { f(x, y) }
fn add(a: &u8, b: &u8) -> u8 { a + b }
fn main() { println!("{}", g(add, &1, &2)); }
