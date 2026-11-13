struct S(u8);
impl Into<u8> for S { fn into(self) -> u8 { self.0 + 1 } }
fn main() { let x: u8 = S(4).into(); println!("{}", x); }
