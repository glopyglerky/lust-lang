// expect: L0403
// rustc E0665: derive(Default) on an enum needs a #[default] variant.
#[derive(Default)]
enum E { A, B }
fn main() {}
