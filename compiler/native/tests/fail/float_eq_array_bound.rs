// expect: L0612
// rustc E0277: an array of floats does not satisfy a `T: Eq` bound.
fn e<T: Eq>(_: T) {}
fn main() { e([1.0f64, 2.0f64]); }
