// expect: L0631
// rustc E0493: the receiver temporary is a drop type destroyed in the const.
struct Owned(u32, String);
impl Owned {
    const fn n(&self) -> u32 {
        self.0
    }
}
const X: u32 = Owned(9, String::new()).n();
fn main() {
    println!("{}", X);
}
