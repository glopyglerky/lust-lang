// expect: L0110
// rustc E0277: [T; N]: Default only up to N = 32.
#[derive(Default)]
struct S([u8; 33]);
fn main() {}
