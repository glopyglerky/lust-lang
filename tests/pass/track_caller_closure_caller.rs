// A #[track_caller] fn's panic! called from a closure reports the call in the closure (W-CHECK-G3).
// panic: track_caller_closure_caller.rs:5:36:
#[track_caller]
fn boom(n: u32) -> ! { panic!("boom {}", n) }
fn main() { let f = |x| if x > 2 { boom(x) } else { x }; println!("{}", f(1)); f(5); }
