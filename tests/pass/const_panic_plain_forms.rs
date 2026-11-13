// The const panic forms rustc allows: a literal message, a plain
// assert, and one `{}` with a &str argument.
const fn chk(x: u8) -> u8 { assert!(x < 9, "bad"); x }
const fn one(m: &str) -> u8 { assert!(!m.is_empty(), "{}", m); 7 }
const C: u8 = chk(8);
const D: u8 = one("m");
const E: u8 = if C > 100 { panic!("too big") } else { 3 };
fn main() { println!("{} {} {}", C, D, E); }
