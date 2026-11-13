const K: usize = 3;
#[derive(Debug)]
enum E { A([u8; K]), B }
fn main() { println!("{:?}", E::A([1, 2, 3])); }
