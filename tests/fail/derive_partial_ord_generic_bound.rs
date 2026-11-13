// expect: L0112
// rustc E0369: derived PartialOrd bounds T: PartialOrd.
#[derive(PartialEq, PartialOrd)]
struct W<T>(T);
#[derive(PartialEq)]
struct N;
fn main() { println!("{}", W(N) < W(N)); }
