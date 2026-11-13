// expect: L0630
// rustc E0080: the method body overflows during const evaluation.
struct W(u32);
impl W {
    const fn add(&self, b: u32) -> u32 {
        self.0 + b
    }
}
const X: u32 = W(u32::MAX).add(1);
fn main() {
    println!("{}", X);
}
