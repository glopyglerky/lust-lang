// expect: L0303
// rustc E0502: the `Box` deref is built in, but `Vec` to slice is not.
fn main() {
    let mut b = Box::new(vec![1, 2]);
    b.swap(0, b.len() - 1);
    println!("{:?}", b);
}
