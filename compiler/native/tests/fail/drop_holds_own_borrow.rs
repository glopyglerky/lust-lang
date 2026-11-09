// expect: L0306
// `tie` makes the borrow of `d` as long as `D`'s own lifetime, and `D`'s
// destructor uses that lifetime when `d` is dropped (rustc E0597).
struct D<'a>(&'a u8);
impl<'a> Drop for D<'a> {
    fn drop(&mut self) {}
}
fn tie<'a>(x: &'a mut D<'a>) -> &'a mut D<'a> {
    x
}
fn main() {
    let z = 1u8;
    let mut d = D(&z);
    let r = tie(&mut d);
    println!("{}", r.0);
}
