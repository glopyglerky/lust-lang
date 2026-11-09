// expect: L0303
// rustc E0502: both halves of split_at_mut borrow `v` mutably until their last use.
fn main() {
    let mut v = vec![1, 2, 3];
    let (a, _b) = v.split_at_mut(1);
    println!("{}", v.len());
    a[0] = 5;
}
