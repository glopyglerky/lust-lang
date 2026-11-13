// expect: L0110
enum E { A(i32), B(i32) }
fn main() {
    match E::B(3) {
        E::A(mut x) | E::B(x) => println!("{}", x),
    }
}
