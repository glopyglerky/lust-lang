use std::thread;
static S: &str = "hi";
fn main() { let r: &'static str = S; let h = thread::spawn(move || r.len()); println!("{}", h.join().unwrap()); }
