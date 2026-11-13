#[derive(Clone, Copy)]
struct F(usize);
impl F {
    const fn n(&self) -> usize {
        self.0
    }
}
const N: usize = F(4).n();
fn main() {
    let a = [1u8; N];
    println!("{}", a.len());
}
