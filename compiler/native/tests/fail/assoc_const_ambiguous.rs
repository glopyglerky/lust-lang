// expect: L0110
struct G<T>(T);
impl G<i32> { const C: u8 = 1; }
impl G<String> { const C: u8 = 2; }
fn main() {
    println!("{}", G::C);
}
