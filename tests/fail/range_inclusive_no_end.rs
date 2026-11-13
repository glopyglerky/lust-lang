// expect: L0001
// rustc: inclusive range with no end
fn main() { let v = [1, 2, 3]; let _ = &v[..=]; }
