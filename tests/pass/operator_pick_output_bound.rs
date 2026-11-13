// Operators record no pick: an instance re-selects the one impl
// coherence allows, and a bound that fixes `Output` is checked at
// the call site (see check/picks.rs). Pin both shapes.
use std::ops::Add;

struct M(u8);

impl Add<M> for M {
    type Output = u16;
    fn add(self, rhs: M) -> u16 {
        self.0 as u16 + rhs.0 as u16
    }
}

fn f<T: Add<T, Output = u16>>(a: T, b: T) -> u16 {
    a + b
}

fn g<T: Add<Output = T> + Copy>(a: &T, b: &T) -> T {
    *a + *b
}

fn main() {
    println!("{}", f(M(200), M(100)));
    println!("{}", g(&20u8, &3u8));
}
