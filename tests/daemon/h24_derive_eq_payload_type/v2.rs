#[derive(PartialEq, Debug)]
enum E { A(bool), B }
fn main() { println!("{} {}", E::A(true) == E::A(true), E::A(true) == E::B); }
