// expect: L0110
enum E { A(i32), B(bool) }
fn main() {
    match E::A(3) {
        E::A(x) | E::B(x) => println!("hi"),
    }
}
