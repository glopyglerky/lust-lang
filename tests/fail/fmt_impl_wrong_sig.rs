// expect: L0110
// rustc E0053: `fmt` must return fmt::Result.
use std::fmt;
struct A;
impl fmt::Display for A {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> bool {
        let _ = f;
        true
    }
}
fn main() {
    println!("{}", A);
}
