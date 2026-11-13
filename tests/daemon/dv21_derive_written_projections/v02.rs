// Written projections that name the toggled type (`Self::Item`, `<S as ToOwned>::Owned` in a param and a field, `Cloned<..>::Item`): the derive delta checks them again (WF on the written type, then normalizes and compares) in place of a full collect. -Clone rejects (ToOwned and Cloned need it).
#[derive(Clone)]
pub struct S { x: u32 }
const ROUTE: u8 = 1;
pub struct It(Option<S>);
impl Iterator for It {
    type Item = S;
    fn next(&mut self) -> Option<Self::Item> { self.0.take() }
}
pub struct Owns { o: <S as ToOwned>::Owned }
fn own(s: <S as ToOwned>::Owned) -> u32 { s.x }
fn cl(s: <std::iter::Cloned<std::slice::Iter<'static, S>> as Iterator>::Item) -> u32 { s.x + 1 }
fn ii(v: Vec<S>) -> <Vec<S> as IntoIterator>::IntoIter { v.into_iter() }
fn main() {
    let _r = ROUTE;
    let mut it = It(Some(S { x: 2 }));
    let a = it.next().map_or(0, |s| s.x);
    let o = Owns { o: S { x: 3 } };
    println!("{} {} {} {} {}", a, o.o.x, own(S { x: 4 }), cl(S { x: 5 }), ii(vec![S { x: 6 }]).count());
}
