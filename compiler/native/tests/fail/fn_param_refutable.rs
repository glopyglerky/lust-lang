// expect: L0110
enum E { A(i32), B(i32), C }
fn f((E::A(x) | E::B(x)): E) -> i32 { x }
fn main() {
    println!("{}", f(E::A(1)));
}
