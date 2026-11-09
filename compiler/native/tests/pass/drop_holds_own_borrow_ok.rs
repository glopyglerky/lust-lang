// The same `&'a mut T<'a>` tie is fine when the drop does not use `'a`:
// a type without drop glue, a `Vec<&'a u8>` (`#[may_dangle]`), or a plain
// reborrow that ends before the drop.
use std::cell::Cell;
struct N<'a>(&'a u8);
struct D<'a>(&'a u8);
impl<'a> Drop for D<'a> {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn tie<'a, T>(x: &'a mut T) -> &'a mut T {
    x
}
fn tie_n<'a>(x: &'a mut N<'a>) -> &'a mut N<'a> {
    x
}
fn tie_v<'a>(x: &'a mut Vec<&'a u8>) -> &'a mut Vec<&'a u8> {
    x
}
fn tie_c<'a>(x: &'a mut Cell<&'a u8>) -> &'a mut Cell<&'a u8> {
    x
}
fn main() {
    let z = 7u8;
    let mut n = N(&z);
    println!("{}", tie_n(&mut n).0);
    let mut v = vec![&z];
    println!("{}", tie_v(&mut v).len());
    let mut c = Cell::new(&z);
    println!("{}", tie_c(&mut c).get());
    let mut d = D(&z);
    println!("{}", tie(&mut d).0);
}
