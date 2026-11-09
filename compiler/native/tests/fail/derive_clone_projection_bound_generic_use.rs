// expect: L0612
// rustc E0277: the derived impl is `impl<T: Iterator + Clone> Clone for
// Items<T> where T::Item: Clone`; `dup` knows only `T: Clone`.
#[derive(Clone)]
struct Items<T: Iterator> {
    x: Vec<T::Item>,
}
#[derive(Clone)]
struct It;
impl Iterator for It {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        None
    }
}
fn need<X: Clone>(_: &X) {}
fn dup<T: Iterator + Clone>(i: &Items<T>) {
    need(i);
}
fn main() {
    let it: Items<It> = Items { x: vec![] };
    dup(&it);
    println!("{}", it.x.len());
}
