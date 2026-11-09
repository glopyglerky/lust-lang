// expect: L0112
// rustc E0277: strip_prefix needs T: PartialEq.
struct N;
fn main() { let v = vec![N]; println!("{}", v.strip_prefix(&[N]).is_some()); }
