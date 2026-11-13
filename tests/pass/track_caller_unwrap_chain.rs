// unwrap in a #[track_caller] fn called from another reports the outermost caller's location (W-CHECK-G3).
// panic: track_caller_unwrap_chain.rs:7:45:
#[track_caller]
fn get(o: Option<u8>) -> u8 { o.unwrap() }
#[track_caller]
fn twice(o: Option<u8>) -> u8 { get(o) }
fn main() { println!("{}", twice(Some(3))); twice(None); }
