#[derive(Debug)]
enum E { B, A(u8) }
fn main() { println!("{:?} {:?}", E::A(1), E::B); }
