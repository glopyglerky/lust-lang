const K: usize = 2;
#[derive(Debug)]
enum E { A([u8; K]), B }
fn main() { println!("{:?}", E::A([1, 2])); }
