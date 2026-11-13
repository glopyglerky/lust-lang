// expect: L0702
// Valid Rust: a #[track_caller] fn as a fn pointer. lust passes the caller's location only on direct calls, so it rejects the value (W-CHECK-G3).
#[track_caller]
fn line() -> u32 { std::panic::Location::caller().line() }
fn main() { let f: fn() -> u32 = line; println!("{}", f() > 0); }
