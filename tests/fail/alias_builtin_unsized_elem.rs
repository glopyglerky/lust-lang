// expect: L0110
// rustc E0277: `Vec<str>` needs `str: Sized` where the path uses it.
type V = Vec<str>;
fn main() { let v = V::new(); println!("{}", v.len()); }
