// expect: L0612
// std has no RangeBounds impl for &Range: pass the range by value.
fn main() {
    let mut v = vec![1, 2, 3, 4];
    let r = 1..3usize;
    let _ = v.drain(&r);
}
