// expect: L0110
// rustc E0308: the bound's target is u64, the fn returns u32.
fn widen<T: Into<u64>>(t: T) -> u32 { let x = t.into(); x }
fn main() { println!("{}", widen(5u8)); }
