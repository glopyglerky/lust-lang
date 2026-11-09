// expect: L0110
// rustc E0308: replacen's count is a usize.
fn main() { let n: i32 = 1; println!("{}", "aa".replacen("a", "b", n)); }
