// expect: L0612
// rustc E0277: an integer literal result has no `Sum<&f64>` impl.
fn main() {
    let v = vec![1.5f64, 2.0];
    let r = Some(&v).map(|v| v.iter().sum()).unwrap_or(0);
    println!("{}", r);
}
