// expect: L0612
// rustc E0277: `Reverse<f64>` is not Ord, because f64 is not.
use std::cmp::Reverse;
fn main() {
    let mut v = vec![1.5, 0.5];
    v.sort_by_key(|&x| Reverse(x));
    println!("{:?}", v);
}
