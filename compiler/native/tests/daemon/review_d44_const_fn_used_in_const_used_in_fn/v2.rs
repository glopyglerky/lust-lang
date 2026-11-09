const fn sq(x: i64) -> i64 { x * x * x }
const A: i64 = sq(3);
fn g() -> i64 { A + 1 }
fn main() { println!("{}", g()); }
