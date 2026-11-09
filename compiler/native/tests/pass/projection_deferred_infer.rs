// A projection blocked on an inference variable at a call becomes a
// fresh variable plus an associated equality the queue settles once
// inference names the type (rustc's normalization to an inference
// variable): the impl arg comes from another argument here.
trait Array {
    type Item;
}
impl<T> Array for [T; 4] {
    type Item = T;
}
impl<T> Array for [T; 2] {
    type Item = T;
}
fn put<A: Array>(_a: A, x: A::Item) -> A::Item {
    x
}
fn same<A: Array, B: Array<Item = A::Item>>(a: A::Item, b: B::Item) -> bool
where
    A::Item: PartialEq,
{
    a == b
}
trait Tr {
    type Out;
    fn out(&self) -> Self::Out;
}
struct S;
struct T;
impl Tr for S {
    type Out = &'static [u8];
    fn out(&self) -> &'static [u8] {
        &D[..2]
    }
}
impl Tr for T {
    type Out = &'static str;
    fn out(&self) -> &'static str {
        "tee"
    }
}
static D: [u8; 3] = [1, 2, 3];
fn take<X: Tr>(_t: X, x: X::Out) -> X::Out {
    x
}
struct H<X: Tr> {
    t: X,
    last: Option<X::Out>,
}
impl<X: Tr> H<X> {
    fn set(&mut self, o: X::Out) {
        self.last = Some(o);
    }
    fn run(&mut self) {
        let o = self.t.out();
        self.set(o);
    }
}
fn main() {
    let r = put([1u8; 4], 9);
    let s = put([0i16; 2], -3);
    println!("{} {}", r.wrapping_mul(30), s.abs());
    println!("{} {}", same::<[u8; 4], [u8; 2]>(3, 3), same::<[i32; 4], [i32; 2]>(1, 2));
    // The deferred item type takes a coercion: `&[u8; 3]` to `&[u8]`.
    let k = take(S, &D);
    println!("{}", k.len());
    let mut a = H { t: S, last: None };
    a.run();
    let mut b = H { t: T, last: None };
    b.run();
    b.set("bee");
    println!("{:?} {:?}", a.last, b.last);
}
