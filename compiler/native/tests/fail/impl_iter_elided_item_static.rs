// expect: L0733
// rustc: lifetime may not live long enough. `Item = &&'static str` (the
// outer lifetime elided to `self`'s) needs `'s: 'static`.
pub struct D<'s> {
    v: Vec<&'s str>,
}
impl<'s> D<'s> {
    fn it(&self) -> impl Iterator<Item = &&'static str> + '_ {
        self.v.iter()
    }
}
fn main() {
    let d = D { v: vec!["a"] };
    println!("{}", d.it().count());
}
