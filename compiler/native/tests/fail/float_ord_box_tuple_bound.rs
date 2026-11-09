// expect: L0612
// rustc E0277: `Box<(f64, u8)>` does not satisfy a `T: Ord` bound.
fn o<T: Ord>(_: T) {}
fn main() { o(Box::new((1.0f64, 2u8))); }
