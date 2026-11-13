// expect: L0110
enum E { A = 1, B = 0, C }
fn main() {
    println!("{}", E::A as i32);
}
