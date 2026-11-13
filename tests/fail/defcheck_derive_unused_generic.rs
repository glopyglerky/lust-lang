// expect: L0112
// rustc E0277: the derived Debug impl is checked at the
// definition, with no use of it.
struct Inner<T>(T);
#[derive(Debug)]
struct Outer<T> {
    inner: Inner<T>,
}
fn main() {}
