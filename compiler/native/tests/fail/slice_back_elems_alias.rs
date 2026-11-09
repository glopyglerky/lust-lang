// expect: L0304
fn main() { let mut v = vec![1, 2, 3]; let s = v.as_mut_slice(); let n = s.len(); let a = &mut s[n - 1]; let b = &mut s[n - 2]; *a = 9; *b = 8; println!("{:?}", v); }
