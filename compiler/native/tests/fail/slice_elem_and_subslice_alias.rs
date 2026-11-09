// expect: L0304
fn main() { let mut v = vec![1, 2, 3]; let s = v.as_mut_slice(); let a = &mut s[0]; let b = &mut s[1..]; *a = 9; b[0] = 8; println!("{:?}", v); }
