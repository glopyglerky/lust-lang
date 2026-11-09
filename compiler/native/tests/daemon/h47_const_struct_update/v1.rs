struct P { a: u32, b: u32 }
const P0: P = P { a: 1, b: 2 };
fn main() { let p = P { a: 9, ..P0 }; println!("{} {}", p.a, p.b); }
