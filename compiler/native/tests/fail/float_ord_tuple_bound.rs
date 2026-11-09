// expect: L0612
// rustc E0277: a tuple with a float does not satisfy a `T: Ord` bound.
fn o<T: Ord>(_: T) {}
fn main() { o((1.0f32, 2u8)); }
