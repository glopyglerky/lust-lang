#[derive(Clone, Copy)]
struct F(u32);
impl F {
    const fn bits(&self) -> u32 {
        self.0
    }
}
const X: u32 = F(4).bits();
fn main() {
    let a = [0u8; F(2).bits() as usize];
    let b = const { F(5).bits() };
    println!("{} {} {}", X, a.len(), b);
}
