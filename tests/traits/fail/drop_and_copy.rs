// rustc-error: E0184
// lust-expect: L0110
#[derive(Clone, Copy)]
struct D(u8);
impl Drop for D { fn drop(&mut self) { } }
fn main() { println!("x"); }
