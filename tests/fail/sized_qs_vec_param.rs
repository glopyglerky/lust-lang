// expect: L0110
// rustc E0277: `Vec<T>` needs a sized `T`; `T: ?Sized` does not give one.
fn f<T: ?Sized>(x: &Vec<T>) -> usize { x.len() }
fn main() {}
