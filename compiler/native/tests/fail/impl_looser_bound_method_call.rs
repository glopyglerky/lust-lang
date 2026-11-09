// expect: L0612
// rustc E0277: a call to a trait impl's method is checked against the
// trait item, so the trait's `T: Clone` holds even though the impl
// method states no bound.
trait Get {
    fn take<T: Clone>(&self, x: T) -> usize;
}
struct A;
impl Get for A {
    fn take<T>(&self, x: T) -> usize {
        std::mem::size_of_val(&x)
    }
}
struct NoClone;
fn main() {
    println!("{}", A.take(NoClone));
}
