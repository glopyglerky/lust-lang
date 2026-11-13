// expect: L0612
// rustc E0277: a float cannot be a `BTreeSet` key (no `Ord`).
fn main() { let mut s = std::collections::BTreeSet::new(); s.insert(1.0f32); }
