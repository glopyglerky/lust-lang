// expect: L0203
// rustc: lifetime may not live long enough. `T: Any` needs `&'a str: 'static`; only a written
// `'static` is the same as the elided form there.
use std::any::Any;
fn f<'a>(x: &'a dyn Any) -> bool {
    x.is::<&'a str>()
}
fn main() {
    println!("{}", f(&1u8));
}
