// rustc-error: E0366
// lust-expect: L0110
struct G<T>(T);
impl Drop for G<u8> { fn drop(&mut self) { println!("G"); } }
fn main() { let _a = G(1u8); }
