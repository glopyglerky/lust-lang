// rustc-error: E0665
// lust-expect: L0403
// ARCH 2.2 derives: derive(Default) on an enum needs one #[default]
// unit variant.
#[derive(Default)]
enum E { A, B }
fn main() {}
