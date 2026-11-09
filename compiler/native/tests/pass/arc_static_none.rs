use std::sync::Arc;
static A: Option<Arc<u8>> = None;
fn main() { println!("{}", A.is_none()); }
