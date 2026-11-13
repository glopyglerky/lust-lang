// expect: L0612
// rustc E0277: a float tuple cannot be a `HashSet` key (no `Eq + Hash`).
fn main() { let mut s = std::collections::HashSet::new(); s.insert((1.0f32, 2.0f32)); }
