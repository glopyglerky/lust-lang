// expect: L0112
// rustc E0277: Inner's written Clone needs `T: Default`, which the derived
// `impl<T: Clone> Clone for Outer<T>` does not give.
#[derive(Clone)]
struct Outer<T> {
    i: (Inner<T>, u8),
}
struct Inner<T>(Option<T>);
impl<T: Clone + Default> Clone for Inner<T> {
    fn clone(&self) -> Self {
        Inner(self.0.clone())
    }
}
fn main() {
    let o: Outer<u8> = Outer { i: (Inner(None), 2) };
    println!("{}", o.i.1);
}
