// expect: L0110
enum E { A(i32), B }
fn main() {
    match E::B {
        E::A(x) | E::B => println!("hi"),
    }
}
