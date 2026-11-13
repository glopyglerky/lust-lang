// expect: L0308
// rustc E0596: as_chunks_mut needs a mutable Vec.
fn main() { let v = vec![1u8, 2, 3, 4]; let (a, _) = v.as_chunks_mut::<2>(); a[0][0] = 9; }
