// expect: L0612
// rustc E0277: a slice of floats does not satisfy a `T: Eq` bound.
fn e<T: Eq>(_: T) {}
fn main() { let v = vec![1.0f32]; e(&v[..]); }
