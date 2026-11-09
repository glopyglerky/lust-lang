// expect: L0110
// rustc E0053: `add` takes `Rhs` (here `S`), not `u8`.
use std::ops::Add;
#[derive(Debug)]
struct S(u8);
impl Add for S {
    type Output = S;
    fn add(self, o: u8) -> S {
        S(self.0 + o)
    }
}
fn main() {
    println!("{:?}", S(1));
}
