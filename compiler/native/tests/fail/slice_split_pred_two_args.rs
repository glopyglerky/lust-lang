// expect: L0612
// rustc E0593: the predicate takes one argument.
fn main() { let v = vec![1, 0]; println!("{}", v.split(|x, y| *x == 0).count()); }
