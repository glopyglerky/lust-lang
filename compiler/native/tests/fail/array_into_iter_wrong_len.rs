// expect: L0110
fn main() { let it: std::array::IntoIter<String, 3> = [String::from("x")].into_iter(); println!("{}", it.len()); }
