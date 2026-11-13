// expect: L0630
// rustc: literal out of range for i32; an unpinned literal in a
// static initializer falls back to i32 like anywhere else.
static C: i128 = { let x = 2147483648; x as i128 };
fn main() { println!("{}", C); }
