// expect: L0110
// rustc E0053: `eq` takes `&Rhs`, not `Rhs`.
struct S(u8);
impl PartialEq for S {
    fn eq(&self, o: S) -> bool {
        self.0 == o.0
    }
}
fn main() {
    println!("{}", S(1).0);
}
