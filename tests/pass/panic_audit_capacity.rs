// A capacity overflow reports where alloc raises it: `repeat`'s
// checked multiply in slice.rs, RawVec's byte limit in raw_vec.
// panic: capacity overflow
fn main() {
    let n = std::env::args().count();
    println!("{}", "ab".repeat(3 * n));
    let v: Vec<u64> = Vec::with_capacity(4 + n);
    println!("{}", v.capacity() >= 4);
    let s = "ab".repeat(usize::MAX / 2 + n);
    println!("{}", s.len());
}
