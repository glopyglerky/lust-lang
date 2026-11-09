// expect: L0104
// rustc E0599: Range<char> is not ExactSizeIterator.
fn main() { println!("{}", ('a'..'z').len()); }
