// rustc-error: E0367
// lust-expect: L0110
struct G<T>(T);
impl<T: Copy> Drop for G<T> { fn drop(&mut self) { println!("G"); } }
fn main() { let _a = G(1u8); }
