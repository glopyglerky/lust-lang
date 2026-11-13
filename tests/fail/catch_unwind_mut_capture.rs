// expect: L0612
// A closure that mutates a captured local through &mut is not UnwindSafe
// (rustc E0277).
struct D(&'static str);
impl Drop for D { fn drop(&mut self) { println!("drop {} {}", self.0, std::thread::panicking()); } }
use std::panic;
fn main() {
    let mut x = 5;
    let r = panic::catch_unwind(|| { x += 1; });
    println!("{:?}", r.is_ok());
}
