// Two chunk_by_mut items alive at once are disjoint (R13).
fn main() { let mut v = vec![1, 1, 2]; let mut it = v.chunk_by_mut(|a, b| a == b); let x = it.next().unwrap(); let y = it.next().unwrap(); x[0] = y[0]; println!("{:?}", v); }
