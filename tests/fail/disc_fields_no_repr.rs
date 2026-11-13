// expect: L0110
enum E { A(u8) = 3, B = 5 }
fn main() {
    let _e = E::B;
    println!("ok");
}
