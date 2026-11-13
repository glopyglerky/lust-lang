// expect: L0106
struct G<T>(T);
impl<T> G<T> { const C: u8 = 1; }
impl G<i32> { const C: u8 = 2; }
fn main() {
    println!("{}", G::<i32>::C);
}
