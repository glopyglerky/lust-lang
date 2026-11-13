// expect: L0303
fn main() { let mut it = [String::from("a"), String::from("b")].into_iter(); let s = it.as_slice(); let _x = it.next(); println!("{:?}", s); }
