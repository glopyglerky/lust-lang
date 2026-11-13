#[derive(Debug)]
enum E { A(u8), B }
fn main() { println!("{:?} {:?}", E::A(1), E::B); }
