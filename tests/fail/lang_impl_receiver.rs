// expect: L0110
// rustc E0053: `clone` takes `&self`, not `&mut self`.
#[derive(Debug)]
struct S(u8);
impl Clone for S {
    fn clone(&mut self) -> S {
        S(self.0)
    }
}
fn main() {
    println!("{:?}", S(1));
}
