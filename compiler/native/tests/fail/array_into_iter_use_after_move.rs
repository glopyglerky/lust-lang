// expect: L0301
fn main() { let a = [String::from("a")]; let _it = a.into_iter(); println!("{:?}", a); }
