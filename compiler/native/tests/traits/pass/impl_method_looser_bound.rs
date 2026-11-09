trait T { fn a<U: Copy>(&self, u: U) -> u8; }
struct X;
impl T for X { fn a<U>(&self, _u: U) -> u8 { 1 } }
fn main() { println!("{}", X.a(5u8)); }
