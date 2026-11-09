mod sub;
use sub::S;
fn main() { let s = S { a: 1, b: 0 }; let t = S { a: 1, b: 0 }; println!("{} {}", s.b, s == t); }
