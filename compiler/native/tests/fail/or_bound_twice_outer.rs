// expect: L0106
enum E { A(i32), B(i32) }
fn main() {
    match (1, E::A(2)) {
        (x, E::A(x) | E::B(x)) => println!("{}", x),
        _ => println!("z"),
    }
}
