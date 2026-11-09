// expect: L0524
// rustc: a fn parameter takes only lint and cfg attributes.
fn f(#[deprecated] x: u8) -> u8 { x }
fn main() { println!("{}", f(1)); }
