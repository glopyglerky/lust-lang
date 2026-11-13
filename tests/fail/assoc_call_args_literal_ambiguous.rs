// expect: L0612
// rustc E0277: the literal falls back to i32, and S has no Conv<i32>.
trait Conv<T> { fn conv(t: T) -> Self; }
struct S { n: u32 }
impl Conv<u8> for S { fn conv(t: u8) -> S { S { n: t as u32 } } }
impl Conv<u16> for S { fn conv(t: u16) -> S { S { n: t as u32 + 1000 } } }
fn main() { println!("{}", S::conv(3).n); }
