// expect: L0641
// Rust prints `Args { inner: [..] }`; lust has no such rendering yet.
fn main() {
    println!("{:?}", std::env::args());
}
