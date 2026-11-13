// expect: L0110
// rustc E0308: `3u8` for a usize param.
fn f<const N: usize>() -> usize { N }
fn main() { println!("{}", f::<{ 3u8 }>()); }
