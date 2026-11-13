// expect: L0612
// rustc E0277: `Vec<f32>` does not satisfy a `T: Eq` bound.
fn e<T: Eq>(_: T) {}
fn main() { e(vec![1.0f32]); }
