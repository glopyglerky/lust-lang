// expect: L0110
// rustc E0747: constant provided when a type was expected.
fn main() { let x: Option<Vec<3>> = None; println!("{}", x.is_none()); }
