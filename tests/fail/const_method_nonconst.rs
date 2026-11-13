// expect: L0631
// rustc E0015: a non-const inherent method in a const item.
struct W(u32);
impl W {
    fn get(&self) -> u32 {
        self.0
    }
}
const X: u32 = W(3).get();
fn main() {
    println!("{}", X);
}
