// expect: L0402
// rustc error: cannot find attribute default without derive(Default).
#[derive(Debug)]
enum E { #[default] A, B }
fn main() { println!("{:?}", E::A); }
