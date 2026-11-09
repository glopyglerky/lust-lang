// expect: L0104
// rustc E0507: into_iter() through a shared reference would move out of it.
fn main() { let r = 0..3u8; let q = &r; let mut it = q.into_iter(); println!("{:?}", it.next()); }
