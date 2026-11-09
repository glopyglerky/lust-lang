// expect: L0309
use std::sync::Arc;
fn main() { let a = Arc::new(5); *a = 6; println!("{}", a); }
