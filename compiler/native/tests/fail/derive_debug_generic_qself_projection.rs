// expect: L0112
// rustc E0277: the derive bounds `T::Item` written as a path that starts
// at `T`, but not `<T as Iterator>::Item`, so this field's Debug is unproven.
#[derive(Debug)]
struct S<T: Iterator> {
    x: Option<<T as Iterator>::Item>,
}
fn main() {
    let s: S<std::ops::Range<u8>> = S { x: None };
    println!("{}", s.x.is_none());
}
