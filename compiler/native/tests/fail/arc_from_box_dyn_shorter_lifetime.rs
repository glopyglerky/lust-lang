// expect: L0733
// rustc: lifetime may not live long enough. `Arc<dyn Speak>` in a signature is
// `'static`, and alloc's `From<Box<T>>` keeps `T`, object lifetime included.
use std::sync::Arc;
trait Speak {
    fn speak(&self) -> String;
}
fn widen<'a>(b: Box<dyn Speak + 'a>) -> Arc<dyn Speak> {
    b.into()
}
struct S;
impl Speak for S {
    fn speak(&self) -> String {
        "s".to_string()
    }
}
fn main() {
    println!("{}", widen(Box::new(S)).speak());
}
