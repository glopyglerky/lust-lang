// expect: L0110
// rustc: `main` may not be #[track_caller] (W-CHECK-G3).
#[track_caller]
fn main() { println!("x"); }
