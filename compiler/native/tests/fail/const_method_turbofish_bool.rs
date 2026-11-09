// expect: L0110
// rustc E0308: `true` for a usize param.
struct S;
impl S { fn f<const N: usize>(&self) -> usize { N } }
fn main() { println!("{}", S.f::<true>()); }
