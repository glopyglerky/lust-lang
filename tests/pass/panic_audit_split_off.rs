// `Vec::split_off` past the length names both numbers.
// panic: `at` split index (is 6) should be <= len (is 3)
fn main() {
    let n = std::env::args().count();
    let mut v = vec![1, 2, 3, 4];
    let t = v.split_off(n + 2);
    println!("{:?} {:?}", v, t);
    let _ = v.split_off(n + 5);
}
