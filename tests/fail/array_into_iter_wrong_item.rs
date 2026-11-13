// expect: L0110
fn main() { let v: Vec<i32> = [String::from("x")].into_iter().collect(); println!("{:?}", v); }
