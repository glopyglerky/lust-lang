// expect: L0403
// rustc error: #[default] only on unit variants.
#[derive(Default)]
enum E { #[default] A(u8), B }
fn main() {}
