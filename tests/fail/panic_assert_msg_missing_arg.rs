// expect: L0001
// rustc: format string arguments do not match.
fn main() {
    let n = std::env::args().count();
    assert!(n > 0, "{}");
}
