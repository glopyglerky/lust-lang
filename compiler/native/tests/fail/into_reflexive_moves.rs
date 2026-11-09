// expect: L0301
// rustc E0382: `into` takes the receiver by value, also for `From<T> for T`.
fn main() { let s = String::new(); let t: String = s.into(); println!("{} {}", t, s.len()); }
