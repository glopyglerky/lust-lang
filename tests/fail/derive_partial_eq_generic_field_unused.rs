// expect: L0112
// rustc E0369: the derived `impl<T: PartialEq> PartialEq for Outer<T>`
// compares two `Option<Box<Inner<T>>>`, and Inner has no PartialEq.
#[derive(PartialEq)]
struct Outer<T> {
    inner: Option<Box<Inner<T>>>,
}
struct Inner<T>(T);
fn main() {
    let o: Outer<u8> = Outer { inner: None };
    println!("{}", o.inner.is_none());
}
