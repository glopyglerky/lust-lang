// expect: L0641
// Valid Rust: `Flatten<slice::Iter<Vec<u8>>>` is double-ended (both levels are). lust keeps
// `Flatten` forward-only, so a written `Rev` over it is a reject.
fn f(x: std::iter::Rev<std::iter::Flatten<std::slice::Iter<'static, Vec<u8>>>>) -> usize {
    x.count()
}
fn main() {
    println!("ok");
}
