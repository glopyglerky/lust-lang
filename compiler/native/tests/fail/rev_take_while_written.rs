// expect: L0641
// rustc E0599: `TakeWhile` is not double-ended, so `Rev<TakeWhile<..>>` is no iterator.
fn f(x: std::iter::Rev<std::iter::TakeWhile<std::slice::Iter<'static, u8>, fn(&&u8) -> bool>>) -> usize {
    x.count()
}
fn main() {
    println!("ok");
}
