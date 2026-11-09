// Fn bounds that name a projection give closures their normalized
// parameter types, and a projection through an impl with a where
// clause normalizes where the clause holds.
trait Array {
    type Item;
}
impl<T> Array for [T; 4] {
    type Item = T;
}
struct P<A: Array> {
    v: Vec<A::Item>,
}
impl<A: Array> P<A> {
    fn each<F: FnMut(&A::Item)>(&self, mut f: F) {
        for x in &self.v {
            f(x);
        }
    }
    fn map_all<F: Fn(A::Item) -> A::Item>(&mut self, f: F)
    where
        A::Item: Copy,
    {
        for x in self.v.iter_mut() {
            *x = f(*x);
        }
    }
}
trait Tr {
    type Out;
}
impl<T: Copy> Tr for Vec<T> {
    type Out = T;
}
fn f(x: <Vec<u8> as Tr>::Out) -> u8 {
    x + 1
}
fn g<T: Copy>(x: <Vec<T> as Tr>::Out) -> T {
    x
}
fn main() {
    let mut p: P<[u32; 4]> = P { v: vec![1, 2, 3] };
    let mut s = 0;
    p.each(|x| s += *x);
    p.map_all(|x| x * 2);
    let y: <Vec<u16> as Tr>::Out = 7;
    println!("{} {:?} {} {} {}", s, p.v, f(4), y, g::<u8>(3));
}
