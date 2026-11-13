// 127 nested const fn frames (plus the item) is exactly at the limit.
const fn rec(n: u32) -> u32 { if n == 0 { 0 } else { 1 + rec(n - 1) } }
static C: u32 = rec(126);
fn main() { println!("{}", C); }
