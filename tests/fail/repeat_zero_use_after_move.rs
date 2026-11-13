// expect: L0301
// rustc E0382: the zero repeat moved `x`.
struct S(u8);
fn main() { let x = S(1); let a = [x; 0]; println!("{}", x.0); }
