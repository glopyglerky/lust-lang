// expect: L0403
// rustc error: multiple declared defaults.
#[derive(Default)]
enum E { #[default] A, #[default] B }
fn main() {}
