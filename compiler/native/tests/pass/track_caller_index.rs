// An index panic in a #[track_caller] fn reports the caller's location (W-CHECK-G3).
// panic: track_caller_index.rs:5:80:
#[track_caller]
fn checked(v: &[u32], i: usize) -> u32 { v[i] }
fn main() { let v = vec![1, 2]; println!("{}", checked(&v, 1)); println!("{}", checked(&v, 5)); }
