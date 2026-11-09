// panic: range end index 5 out of range for slice of length 3
// `copy_within` with an out-of-range source panics at core's
// slice-index site, like `drain` (tf-slice).
fn main() {
    let n = std::env::args().count();
    let mut v = vec![0, 1, 2];
    println!("start {n}");
    v.copy_within(0..5, 0);
    println!("{:?}", v);
}
