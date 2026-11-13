// expect: L0104
struct W<T>(T);
impl W<u8> {
    fn get(&self) -> u8 {
        self.0
    }
}
fn main() {
    println!("{}", W(300u32).get());
}
