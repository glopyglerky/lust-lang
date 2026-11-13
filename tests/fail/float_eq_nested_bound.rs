// expect: L0612
// rustc E0277: a float nested in a tuple of containers breaks `Eq`.
fn e<T: Eq>(_: T) {}
fn main() { e((vec![(1u8, 1.0f32)], 2u8)); }
