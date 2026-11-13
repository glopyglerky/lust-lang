// expect: L0112
// rustc E0277: `Vec<Inner<T>>: Debug` needs `Inner<T>: Debug` in the
// derived impl of Outer, used or not.
#[derive(Debug)]
struct Outer<T> {
    inner: Vec<Inner<T>>,
}
struct Inner<T>(T);
fn main() {
    let o: Outer<u8> = Outer { inner: vec![] };
    println!("{}", o.inner.len());
}
