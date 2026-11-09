// expect: L0110
// RangeBounds<i32> is not RangeBounds<usize>.
fn main() {
    let mut v = vec![1, 2, 3, 4];
    let _ = v.drain(0i32..2i32);
}
