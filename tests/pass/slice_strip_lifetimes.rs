// The strip result borrows the receiver only, as core's elided signature says (R13).
fn f<'a>(s: &'a [u8], p: &[u8]) -> &'a [u8] { s.strip_prefix(p).unwrap_or(s) }
fn g<'a>(s: &'a mut Vec<u8>) -> &'a [u8] { s.strip_suffix(b"\n").unwrap_or(&[]) }
fn main() { let mut v = b"x\n".to_vec(); println!("{:?} {:?}", f(b"ab", b"a"), g(&mut v)); }
