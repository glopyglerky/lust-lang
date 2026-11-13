// expect: L0104
// rustc E0658: `Option::zip_with` is unstable in 1.99.
fn main() {
    let _ = Some(1).zip_with(Some(2), |a, b| a + b);
}
