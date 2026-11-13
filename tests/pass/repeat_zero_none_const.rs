// A unit variant operand makes no temporary, so a zero repeat of it
// in a const item drops nothing.
static Y: [Option<String>; 0] = [None; 0];
fn main() { println!("{}", Y.len()); }
