// A derived impl on a generic Adt is checked at the definition under the
// derive's bounds (fx-derivegen): Outer<T> derives Debug over Vec<Inner<T>>
// and Option<Box<Inner<T>>> and nothing formats an Outer. Inner's Debug goes
// from a derive to written impls with several bounds and back.
const _ROUTE: u8 = 0;
use std::fmt;
#[derive(Debug, Clone)]
struct Outer<T> {
    v: Vec<Inner<T>>,
    o: Option<Box<Inner<T>>>,
    n: u8,
}
fn main() {
    let o: Outer<u8> = Outer { v: vec![Inner(1)], o: None, n: 3 };
    let c = o.clone();
    println!("{} {}", o.n, c.v.len());
}
#[derive(Clone)]
struct Inner<T>(T);
impl<T> fmt::Debug for Inner<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "I")
    }
}
