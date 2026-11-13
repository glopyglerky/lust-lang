// expect: L0110
// rustc E0053: `drop` takes `&mut self`.
struct S(u8);
impl Drop for S {
    fn drop(&self) {
        println!("drop {}", self.0);
    }
}
fn main() {
    let _s = S(1);
}
