// expect: L0612
// rustc E0277: no impl of From<O2> for S.
struct O { v: u8 }
struct O2 { v: u8 }
struct S { n: u8 }
impl From<O> for S { fn from(o: O) -> S { S { n: o.v } } }
fn main() { println!("{}", S::from(O2 { v: 1 }).n); }
