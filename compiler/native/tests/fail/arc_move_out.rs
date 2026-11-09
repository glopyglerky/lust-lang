// expect: L0305
use std::sync::Arc;
fn main() { let a = Arc::new(String::from("s")); let s = *a; println!("{}", s); }
