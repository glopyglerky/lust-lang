// Slice range checks in Rust 1.99 order: start, then end, then order.
// panic: range end index 7 out of range for slice of length 4
fn main() {
    let n = std::env::args().count();
    let v = vec![1, 2, 3, 4];
    println!("{:?} {:?}", &v[n..n + 2], &v[n + 3..]);
    let s = &v[..];
    println!("{:?}", &s[n..=n + 2]);
    println!("{:?}", &v[n..n + 6]);
}
