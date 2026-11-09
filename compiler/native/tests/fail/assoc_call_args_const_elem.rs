// expect: L0612
// rustc E0277: [u16; 2] is no [u8; N].
struct S { n: usize }
impl<const N: usize> From<[u8; N]> for S { fn from(_a: [u8; N]) -> S { S { n: N } } }
fn main() { println!("{}", S::from([1u16, 2]).n); }
