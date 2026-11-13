const fn k() -> isize { 3 }
enum E { A = k(), B }
fn main() { println!("{} {}", E::A as isize, E::B as isize); }
