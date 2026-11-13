// expect: L0733
// rustc E0621: the strip result borrows the receiver, not the pattern.
fn g<'a>(s: &[u8], p: &'a [u8]) -> &'a [u8] { s.strip_prefix(p).unwrap_or(p) }
fn main() { println!("{:?}", g(b"ab", b"a")); }
