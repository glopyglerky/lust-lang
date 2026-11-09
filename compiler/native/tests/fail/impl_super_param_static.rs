// expect: L0617
// rustc E0310: the selected `Tk` impl needs `T: 'static`, which the owing impl's type
// parameter does not have.
trait Tk<A> {}
struct S;
impl<T: 'static> Tk<T> for S {}
trait Sup<A>: Tk<A> {}
impl<T> Sup<T> for S {}
fn main() {
    println!("x");
}
