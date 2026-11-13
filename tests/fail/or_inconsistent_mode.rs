// expect: L0110
enum E { A(i32), B(i32) }
fn main() {
    let e = E::B(3);
    match e {
        E::A(ref x) | E::B(x) => println!("{}", x),
    }
}
