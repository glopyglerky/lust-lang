// expect: L0304
// rustc E0499: the peeked reference borrows the Peekable mutably.
fn main() { let src = vec![1u32, 2]; let mut f = src.iter().peekable(); let p = f.peek().unwrap(); f.next(); println!("{}", **p); }
