// expect: L0110
// rustc E0308: with no binding, I::Item is not u8.
fn first<I: IntoIterator>(i: I) -> u8 { let x: u8 = i.into_iter().next().unwrap(); x }
fn main() { println!("{}", first(vec![1u8])); }
