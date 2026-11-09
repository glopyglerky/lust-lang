// expect: L0612
// rustc E0277: `T: Clone` is not in the env of `f`.
fn f<T: 'static>(a: <std::iter::Cloned<std::slice::Iter<'static, T>> as Iterator>::Item) -> T {
    a
}
fn main() {
    println!("{}", f(1u8));
}
