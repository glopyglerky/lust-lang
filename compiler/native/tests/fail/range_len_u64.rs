// expect: L0104
// rustc E0599: Range<u64> is not ExactSizeIterator (core's range_exact_iter_impl! list).
fn main() { let r = 0u64..5; println!("{}", r.len()); }
