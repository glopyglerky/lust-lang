// expect: L0110
// rustc E0308: a never-read const body is type checked in
// full; the dead branch has the wrong type.
const BAD: u8 = if true { 1 } else { false };
fn main() {}
