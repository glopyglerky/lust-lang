// expect: L0733
// rustc E0521: dedup_by's references are higher-ranked; one cannot escape the closure.
fn main() { let mut v = vec![String::new()]; let mut keep = None; v.dedup_by(|a, _b| { keep = Some(a); false }); println!("{:?}", keep); }
