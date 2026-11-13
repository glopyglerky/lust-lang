// expect: L0110
// rustc: a repeat of a non-Copy static value would move out of it.
struct S(u8);
static ST: S = S(1);
static Y: [S; 1] = [ST; 1];
fn main() { println!("{}", Y.len()); }
