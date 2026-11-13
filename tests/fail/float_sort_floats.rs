// expect: L0612
// rustc E0277: `sort` needs `Ord`, which floats do not implement.
fn main() { let mut v = vec![2.0f32, 1.0f32]; v.sort(); }
