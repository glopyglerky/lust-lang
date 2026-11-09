// expect: L0612
// rustc E0277: `Y: PartialEq` does not prove `(Y, u8): Eq`.
fn e<T: Eq>(_: T) {}
fn f<Y: PartialEq>(y: Y) { e((y, 1u8)); }
fn main() { f(3u8); }
