// expect: L0110
enum E { A = 1, B = 1 }
fn main() {
    println!("{}", E::A as i32);
}
