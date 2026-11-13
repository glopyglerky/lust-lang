// expect: L0612
// rustc E0277: `Option<i32>: Sum<i32>` does not hold; only `Sum<Option<U>>`.
fn main() {
    let s: Option<i32> = vec![1, 2].into_iter().sum();
    println!("{:?}", s);
}
