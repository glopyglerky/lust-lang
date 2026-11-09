// expect: L0612
// rustc E0277: a path call through a trait impl is checked against the
// trait item too (`<A as Get>::take`, `Get::take`, `A::take`).
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
    println!("{}", <A as Get>::take(&A, NoClone));
}
