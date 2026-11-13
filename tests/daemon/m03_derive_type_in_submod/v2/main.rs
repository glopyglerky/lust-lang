mod sub;
use sub::S;
fn main() { let s = S { a: 1, b: false }; let t = S { a: 1, b: false }; println!("{} {}", s.b, s == t); }
