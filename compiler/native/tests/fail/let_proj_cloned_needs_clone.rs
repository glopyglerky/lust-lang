// expect: L0612
// rustc E0277: a `let` annotation through `Cloned<I>` needs `A: Clone` (a program with no
// generics of its own).
#[derive(Debug)]
struct A(u8);
fn main() {
    let a: <std::iter::Cloned<std::slice::Iter<'static, A>> as Iterator>::Item = A(1);
    println!("{:?}", a);
}
