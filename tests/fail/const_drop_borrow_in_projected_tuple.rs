// expect: L0631
// rustc E0493: the tuple temporary is projected, not extended.
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {}
}
const X: &D = (&D(1), 5).0;
fn main() { println!("{}", X.0); }
