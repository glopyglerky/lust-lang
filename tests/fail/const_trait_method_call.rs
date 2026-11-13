// expect: L0630
// rustc E0015: trait methods are never const on stable.
trait Tr {
    fn v(&self) -> u32;
}
struct W(u32);
impl Tr for W {
    fn v(&self) -> u32 {
        self.0
    }
}
const X: u32 = W(3).v();
fn main() {
    println!("{}", X);
}
