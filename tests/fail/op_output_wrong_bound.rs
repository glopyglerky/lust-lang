// expect: L0110
// rustc E0271: the bound fixes `Output = u16`, the impl gives `M`.
use std::ops::Add;

struct M(u8);

impl Add for M {
    type Output = M;
    fn add(self, rhs: M) -> M {
        M(self.0 + rhs.0)
    }
}

fn f<T: Add<Output = u16>>(a: T, b: T) -> u16 {
    a + b
}

fn main() {
    println!("{}", f(M(1), M(2)));
}
