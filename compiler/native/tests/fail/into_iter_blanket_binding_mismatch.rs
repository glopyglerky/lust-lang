// expect: L0110
// rustc E0271: D yields u8, the bound asks for u32.
struct C { n: u32 }
impl Iterator for C { type Item = u32; fn next(&mut self) -> Option<u32> { if self.n < 3 { self.n += 1; Some(self.n) } else { None } } }
struct D { n: u8 }
impl Iterator for D { type Item = u8; fn next(&mut self) -> Option<u8> { if self.n < 2 { self.n += 1; Some(self.n) } else { None } } }
fn g<I: IntoIterator<Item = u32>>(i: I) -> u32 { let mut s = 0; for x in i { s += x; } s }
fn main() { println!("{}", g(D { n: 0 })); }
