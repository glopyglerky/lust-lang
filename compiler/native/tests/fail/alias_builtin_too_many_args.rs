// expect: L0107
// rustc E0107: the alias has one parameter.
type VS<T> = Vec<T>;
fn main() { let v = VS::<u8, u8>::new(); println!("{:?}", v); }
