// expect: L0612
// rustc E0277: `x.eq(&x)` on a by-value iterator is `Iterator::eq`
// (found before the autoref'd `PartialEq::eq`), and `&I` is no
// IntoIterator.
#[derive(PartialEq)]
struct I(u8);
impl Iterator for I {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        None
    }
}
fn main() {
    let x = I(1);
    println!("{:?}", x.eq(&x));
}
