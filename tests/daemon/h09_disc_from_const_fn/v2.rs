const fn k() -> isize { 10 }
enum E { A = k(), B }
fn main() { println!("{} {}", E::A as isize, E::B as isize); }
