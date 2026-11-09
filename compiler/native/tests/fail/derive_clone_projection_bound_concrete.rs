// expect: L0612
// rustc E0277: `Items<It>: Clone` needs `<It as Iterator>::Item: Clone`
// (the derive bounds `T::Item`), and NC is not Clone.
#[derive(Clone)]
struct Items<T: Iterator> {
    x: Vec<T::Item>,
}
#[derive(Clone)]
struct It;
struct NC;
impl Iterator for It {
    type Item = NC;
    fn next(&mut self) -> Option<NC> {
        None
    }
}
fn need<X: Clone>(_: &X) {}
fn main() {
    let it: Items<It> = Items { x: vec![] };
    need(&it);
    println!("{}", it.x.len());
}
