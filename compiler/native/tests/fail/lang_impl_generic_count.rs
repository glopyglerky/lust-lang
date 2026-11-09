// expect: L0110
// rustc E0049: `clone` has no type parameters.
#[derive(Debug)]
struct S(u8);
impl Clone for S {
    fn clone<T>(&self) -> S {
        S(self.0)
    }
}
fn main() {
    println!("{:?}", S(1));
}
