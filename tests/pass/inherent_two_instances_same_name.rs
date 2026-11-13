struct W<T>(T);
impl W<u8> {
    fn which(&self) -> u8 {
        1
    }
}
impl W<u16> {
    fn which(&self) -> u8 {
        2
    }
}
fn main() {
    println!("{} {}", W(0u8).which(), W(0u16).which());
}
