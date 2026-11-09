// expect: L0617
// rustc E0309: a body annotation through a qualified path names `'a`; `T: 'a` is not implied.
fn f<'a, 'b, T: Clone>(_x: &'b T, _y: &'a u8) -> usize {
    let v: Vec<<std::iter::Cloned<std::slice::Iter<'a, T>> as Iterator>::Item> = Vec::new();
    v.len()
}
fn main() {
    println!("{}", f(&1u8, &2));
}
