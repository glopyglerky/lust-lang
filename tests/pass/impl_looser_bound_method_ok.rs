// An impl method with a looser bound than its trait item: calls that meet
// the trait item's bound run the impl's body.
trait Get {
    fn take<T: Clone>(&self, x: T) -> usize;
    fn pair<T: Clone, U: Copy>(&self, x: T, y: U) -> usize;
}
struct A;
impl Get for A {
    fn take<T>(&self, x: T) -> usize {
        std::mem::size_of_val(&x)
    }
    fn pair<T, U>(&self, _x: T, _y: U) -> usize {
        2
    }
}
fn main() {
    println!("{} {} {}", A.take(1u8), (&A).take(1u64), A.pair(String::new(), 1u8));
}
