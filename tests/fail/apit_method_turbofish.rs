// expect: L0107
// rustc E0107: an `impl Trait` argument takes no written generic argument.
use std::fmt::Display;
struct S;
impl S {
    fn m(&self, x: impl Display) -> String {
        format!("[{}]", x)
    }
}
fn main() {
    println!("{}", S.m::<u8>(1));
}
