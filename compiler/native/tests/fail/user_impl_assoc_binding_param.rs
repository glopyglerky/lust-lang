// expect: L0612
// rustc E0277: the impl binds `T` only through `I: Iterator<Item = T>`; its `T: Clone`
// must hold for `<W<vec::IntoIter<A>> as Tr>::Out`.
struct A(u8);
trait Tr {
    type Out;
}
struct W<I>(I);
impl<I: Iterator<Item = T>, T: Clone> Tr for W<I> {
    type Out = T;
}
fn f(a: <W<std::vec::IntoIter<A>> as Tr>::Out) -> u8 {
    a.0
}
fn main() {
    println!("{}", f(A(1)));
}
