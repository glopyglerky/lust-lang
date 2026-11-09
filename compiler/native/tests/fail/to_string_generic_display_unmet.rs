// expect: L0612
// rustc E0599: `G<N>: Display` needs `N: Display`, so `G(N)` has no `to_string`.
use std::fmt;
struct G<T>(T);
impl<T: fmt::Display> fmt::Display for G<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "G{}", self.0)
    }
}
struct N;
fn main() {
    println!("{}", G(N).to_string());
}
