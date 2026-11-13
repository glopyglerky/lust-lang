// expect: L0612
// rustc E0277: a reference to a float tuple does not satisfy a `T: Eq` bound.
fn e<T: Eq>(_: T) {}
fn main() { e(&(1.0f32, 2u8)); }
