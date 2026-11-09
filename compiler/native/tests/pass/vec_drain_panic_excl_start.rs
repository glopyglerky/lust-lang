// panic: range end index 2 out of range for slice of length 3
fn main() {
    let mut v = vec![1, 2, 3];
    println!("before");
    let _ = v.drain((std::ops::Bound::Excluded(2), std::ops::Bound::Excluded(2)));
    println!("after");
}
