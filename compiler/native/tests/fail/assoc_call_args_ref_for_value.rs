// expect: L0612
// rustc E0277: only From<O>, no coercion into the trait input.
struct O { v: u8 }
struct S { n: u8 }
impl From<O> for S { fn from(o: O) -> S { S { n: o.v } } }
fn main() { let o = O { v: 1 }; println!("{}", S::from(&o).n); }
