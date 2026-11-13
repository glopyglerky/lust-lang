// expect: L0733
// rustc E0521: the collected `&u32`s borrow from `v`, so they are not `'static`.
fn bad<'a>(v: &'a Vec<u32>) -> Vec<&'static u32> { Vec::from_iter(v) }
fn main() { let v = vec![1]; println!("{:?}", bad(&v)); }
