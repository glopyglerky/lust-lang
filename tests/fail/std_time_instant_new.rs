// expect: L0104
// rustc E0599: `Instant` has no `new`.
use std::time::Instant;
fn main() {
    let t = Instant::new();
    println!("{:?}", t);
}
