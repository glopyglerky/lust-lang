// expect: L0612
// rustc E0277: `Cloned<I>: Iterator` needs `T: Clone`, where `T` is bound only through
// `I: Iterator<Item = &T>` (w-sig-derive 02:24).
#[derive(Debug)]
struct A(u8);
fn p7_cl(a: <std::iter::Cloned<std::slice::Iter<'static, A>> as Iterator>::Item) -> u8 {
    a.0
}
fn main() {
    println!("{}", p7_cl(A(1)));
}
