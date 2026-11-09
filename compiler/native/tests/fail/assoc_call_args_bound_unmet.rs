// expect: L0612
// rustc E0277: NC is not Copy, so the generic impl does not apply.
struct NC;
struct S { n: usize }
impl<T: Copy> From<(T, T)> for S { fn from(_a: (T, T)) -> S { S { n: 2 } } }
fn main() { println!("{}", S::from((NC, NC)).n); }
