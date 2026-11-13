// An impl's trait obligations hold by regions: a `'static` header, a `'a: 'static` fact,
// one lifetime the selected impl names twice, the implied bound of `&'a &'b u32`, and a
// type parameter with `T: 'static`.
trait Tk<A> {
    fn tk(&self) -> u32 {
        1
    }
}
struct S;
struct R;
struct Q;
struct P;
impl Tk<&'static u32> for S {}
impl<'x> Tk<(&'x u32, &'x u32)> for R {}
impl<'x, 'y: 'x> Tk<&'x &'y u32> for Q {}
impl<T: 'static> Tk<T> for P {}
trait Sup<A>: Tk<A> {
    fn sup(&self) -> u32 {
        self.tk() + 1
    }
}
impl Sup<&'static u32> for S {}
impl<'a> Sup<(&'a u32, &'a u32)> for R {}
impl<'a, 'b> Sup<&'a &'b u32> for Q {}
impl<T: 'static> Sup<T> for P {}
fn main() {
    println!("{} {} {} {}", Sup::<&u32>::sup(&S), Sup::<(&u32, &u32)>::sup(&R), Sup::<&&u32>::sup(&Q), Sup::<u8>::sup(&P));
}
