// expect: L0104
// rustc E0599: write_char comes from fmt::Write, which is not in scope.
use std::fmt;
struct A;
impl fmt::Display for A {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('x')
    }
}
fn main() {
    println!("{}", A);
}
