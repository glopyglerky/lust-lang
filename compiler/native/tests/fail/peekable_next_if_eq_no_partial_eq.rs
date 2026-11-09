// expect: L0112
// rustc E0277: next_if_eq needs Item: PartialEq.
struct N; fn main() { let mut p = vec![N].into_iter().peekable(); println!("{}", p.next_if_eq(&N).is_some()); }
