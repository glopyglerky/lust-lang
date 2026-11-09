const _R: u8 = 0;
const fn cf() -> i32 { "bad" }
const C: i32 = cf();
const D: i32 = C + 1;
fn main() { println!("{} {}", C, D); }
