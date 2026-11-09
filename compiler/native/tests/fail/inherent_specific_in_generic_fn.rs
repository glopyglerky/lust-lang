// expect: L0104
struct W<T>(T);
impl W<u8> {
    fn special(&self) -> u8 {
        self.0 + 1
    }
}
fn g<T>(w: &W<T>) -> u8 {
    w.special()
}
fn main() {
    println!("{}", g(&W(4u8)));
}
