// expect: L0733
// rustc: lifetime may not live long enough. `W<'static, &u8>` pins the
// first field to `'static`; the closure puts its argument there.
#[derive(Debug)]
pub struct W<'a, X>(&'a u8, X);
fn main() {
    let f: Box<dyn Fn(&u8, usize) -> (W<'static, &u8>, &u8)> = Box::new(|a, _| (W(a, a), a));
    let y = 2u8;
    println!("{:?}", f(&y, 0));
}
