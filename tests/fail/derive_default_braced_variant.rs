// expect: L0403
// rustc error: #[default] only on unit variants (braced, even empty, is not unit).
#[derive(Default)]
enum E { A, #[default] B {} }
fn main() {}
