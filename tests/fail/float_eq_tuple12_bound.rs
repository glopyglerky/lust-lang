// expect: L0612
// rustc E0277: one float in a 12-tuple breaks `Eq`.
fn e<T: Eq>(_: T) {}
fn main() { e((1.0f32, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8)); }
