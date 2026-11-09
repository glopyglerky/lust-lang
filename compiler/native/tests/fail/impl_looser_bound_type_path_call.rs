// expect: L0612
// rustc E0277: `A::take` is the trait impl's method, checked against the
// trait item's `T: Copy`.
trait Get {
    fn take<T>(&self, x: T) -> usize
    where
        T: Copy;
}
struct A;
impl Get for A {
    fn take<T>(&self, _x: T) -> usize {
        1
    }
}
fn main() {
    println!("{}", A::take(&A, String::new()));
}
