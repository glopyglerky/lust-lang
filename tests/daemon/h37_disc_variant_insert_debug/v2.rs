#[derive(Debug)]
enum E { Z, A, B }
fn main() { println!("{:?} {} {}", E::B, E::A as u8, E::B as u8); }
