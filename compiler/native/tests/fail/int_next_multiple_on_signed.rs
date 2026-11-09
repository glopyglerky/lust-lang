// expect: L0104
// rustc E0599: `next_multiple_of` exists only on unsigned integers.
fn main() { println!("{}", 37i32.next_multiple_of(5)); }
