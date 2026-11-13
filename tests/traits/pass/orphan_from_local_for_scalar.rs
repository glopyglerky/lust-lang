struct S(u8);
impl From<S> for u32 { fn from(s: S) -> u32 { s.0 as u32 * 2 } }
fn main() { let x: u32 = S(4).into(); println!("{}", x); }
