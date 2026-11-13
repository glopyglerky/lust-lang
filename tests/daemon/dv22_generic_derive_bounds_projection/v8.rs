// Derive bounds of a generic Adt with a T::Item field (fx-derivegen): the
// derive bounds T and T::Item per trait; Inner<T>'s impls go and come back,
// Copy needs Clone and Copy fields, and a generic fn sees only the record.
const _ROUTE: u8 = 0;
#[derive(PartialEq, Eq, Hash)]
struct Inner<T>(Option<T>);
#[derive(Clone, PartialEq)]
struct Out<T: Iterator> {
    x: Vec<T::Item>,
    i: (Inner<T>, u8),
}
fn need<X: Clone>(_: &X) {}
fn via<T: Iterator + Clone>(o: &Out<T>)
where
    T::Item: Clone,
{
    need(o);
}
#[derive(Clone)]
struct It;
impl Iterator for It {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        None
    }
}
fn main() {
    let o: Out<It> = Out { x: vec![1], i: (Inner(None), 2) };
    println!("{} {}", o.x.len(), o.i.1);
}
