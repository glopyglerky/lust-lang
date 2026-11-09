// expect: L0612
// rustc E0277: an array of floats does not satisfy a `T: Hash` bound.
fn h<T: std::hash::Hash>(_: T) {}
fn main() { h([1.0f32, 2.0f32]); }
