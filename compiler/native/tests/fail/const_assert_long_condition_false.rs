// expect: L0630
// rustc E0080: a false no-message const assert! is a compile error, also
// when its condition is too long for lust to print rustc's text.
const A: u32 = 1;
const _: () = assert!(A == 1 && A + 1 == 2 && A + 2 == 3 && A + 3 == 4 && A + 4 == 5 && A + 5 == 7);
fn main() {}
