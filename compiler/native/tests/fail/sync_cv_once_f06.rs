// expect: L0303
use std::sync::OnceLock;
fn main() { let o: OnceLock<String> = OnceLock::new(); let r = o.get_or_init(|| String::from("a")); o.take(); println!("{}", r); }
