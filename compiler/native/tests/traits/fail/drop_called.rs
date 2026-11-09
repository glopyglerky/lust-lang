// rustc-error: E0040
// lust-expect: L0110
struct D(u8);
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); } }
fn main() { let mut d = D(1); d.drop(); }
