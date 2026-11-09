// expect: L0307
// rustc E0515: the closure returns a borrow of its own local, not of the argument.
pub enum L<'a> {
    A(&'a u8),
}
fn main() {
    let f: for<'a> fn(&'a u8) -> Option<L<'a>> = |x| {
        let t = *x;
        Some(L::A(&t))
    };
    let v = 3u8;
    if let Some(L::A(r)) = f(&v) {
        println!("{r}");
    }
}
