// expect: L0112
// rustc E0277: the derived `impl<T: Debug> Debug for Outer<T>` needs
// `Inner<T>: Debug`, and Inner has no Debug impl. rustc checks the derived
// impl at the definition, though nothing formats an Outer.
#[derive(Debug)]
struct Outer<T> {
    inner: Inner<T>,
}
struct Inner<T>(T);
fn main() {
    let o: Outer<u8> = Outer { inner: Inner(1) };
    println!("{}", o.inner.0);
}
