// A trait reference's binding may name a supertrait's
// associated type (R24): `ExactSizeIterator<Item = ..>` and a
// user `Sub<Out = ..>` over `trait Sub: Sup`.
struct V { v: Vec<i32> }
impl V {
    fn b(&self) -> impl ExactSizeIterator<Item = &i32> { self.v.iter() }
}
trait Mid: Iterator {}
fn use_mid<I: ExactSizeIterator<Item = i32>>(it: I) -> i32 { it.sum() }

trait Sup { type Out; }
trait Sub: Sup {}
struct W;
impl Sup for W { type Out = i32; }
impl Sub for W {}
fn use_sub<T: Sub<Out = i32>>(x: T) -> i32 { 7 }
fn opaque_sub() -> impl Sub<Out = i32> { W }
fn main() {
    let v = V { v: vec![1,2,3] };
    let mut s = 0; for x in v.b() { s += x; }
    println!("{} {} {} {}", s, v.b().len(), use_mid(vec![4,5].into_iter()), use_sub(W));
    let _o = opaque_sub();
    println!("ok");
}
