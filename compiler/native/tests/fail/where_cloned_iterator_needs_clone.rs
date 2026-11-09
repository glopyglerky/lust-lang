// expect: L0612
// rustc E0277: a where clause over no param must hold; `Cloned<I>: Iterator` needs `A: Clone`.
#[derive(Debug)]
struct A(u8);
fn need() -> u8
where
    std::iter::Cloned<std::slice::Iter<'static, A>>: Iterator,
{
    1
}
fn main() {
    println!("{}", need());
}
