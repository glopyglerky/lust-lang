// expect: L0110
// rustc E0277: trait `Self` is `?Sized`, so `Option<Self>` is not
// well-formed without a `Self: Sized` bound.
trait T { fn m(&self) -> Option<Self>; }
impl T for u8 { fn m(&self) -> Option<Self> { Some(*self) } }
fn main() { println!("{:?}", 3u8.m()); }
