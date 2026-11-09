// expect: L0303
// rustc E0499: two live mutable chunk views of v.
fn main() { let mut v = vec![1u8, 2, 3, 4]; let (a, b) = v.as_chunks_mut::<2>(); let (c, d) = v.as_chunks_mut::<2>(); a[0][0] = c[0][0]; }
