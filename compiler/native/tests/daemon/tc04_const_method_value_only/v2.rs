#[derive(Clone, Copy)]
struct F(u32);
impl F {
    const fn bits(&self) -> u32 {
        self.0 * 3
    }
}
const X: u32 = F(4).bits();
fn main() {
    println!("{}", X);
}
