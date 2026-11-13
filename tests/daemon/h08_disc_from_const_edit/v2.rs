const K: isize = 10;
enum E { A = K, B }
fn main() { println!("{} {}", E::A as isize, E::B as isize); }
