// expect: L0631
// rustc E0015: a formatting message in an inline const block.
const N: u32 = 2;
fn main() { const { assert!(N < 3, "n={}", N); }; println!("ok"); }
