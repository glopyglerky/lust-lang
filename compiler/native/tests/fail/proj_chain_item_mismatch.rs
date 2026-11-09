// expect: L0612
// rustc E0271: `Chain<A, B>: Iterator` needs `B: Iterator<Item = A::Item>`.
fn f(a: <std::iter::Chain<std::slice::Iter<'static, u8>, std::slice::Iter<'static, u16>> as Iterator>::Item) -> u8 {
    *a
}
fn main() {
    println!("{}", f(&1));
}
