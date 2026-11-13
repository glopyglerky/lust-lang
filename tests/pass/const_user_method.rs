struct P(u32);
impl P {
    const fn get(self) -> u32 {
        self.0
    }
}
const N: u32 = P(9).get();
fn main() {
    println!("{N}");
}
