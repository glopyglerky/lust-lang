// Concrete projections in plain signatures, fn values, inherent
// methods, closures, and let annotations normalize through the impl
// (collect normalizes every signature once the impl index is done).
trait Tr {
    type Out;
    fn mk(&self) -> Self::Out;
}
struct A;
impl Tr for A {
    type Out = u8;
    fn mk(&self) -> u8 {
        7
    }
}
struct Wrap<T>(T);
impl<T: Copy> Tr for Wrap<T> {
    type Out = (T, T);
    fn mk(&self) -> (T, T) {
        (self.0, self.0)
    }
}
trait Array {
    type Item;
}
impl<T> Array for [T; 4] {
    type Item = T;
}

fn needs() -> <A as Tr>::Out {
    8
}
fn take(x: <A as Tr>::Out) -> u8 {
    x + 1
}
struct B;
impl B {
    fn m(&self) -> <A as Tr>::Out {
        3
    }
    fn n(&self, v: <A as Tr>::Out) -> u8 {
        v * 2
    }
    fn via(&self) -> <A as Tr>::Out {
        A.mk()
    }
}
fn twice<X: Tr>(x: &X) -> (X::Out, X::Out) {
    (x.mk(), x.mk())
}
fn keep<T: Array>(x: T::Item) -> Vec<T::Item> {
    let y: T::Item = x;
    let v: Vec<T::Item> = vec![y];
    v
}
fn plus<T: Array<Item = u32>>(x: T::Item) -> u32 {
    let y: T::Item = x;
    y + 1
}

fn main() {
    println!("{} {}", needs(), take(5));
    let x: u8 = needs();
    let f: fn() -> u8 = needs;
    println!("{} {} {}", x + 1, f(), B.m() + 1u8);
    println!("{} {}", B.n(4), B.via() + 1);
    let w = Wrap(3u8);
    let (a, b) = w.mk();
    let ((c, _), (_, d)) = twice(&w);
    let o: <Wrap<u8> as Tr>::Out = (1, 2);
    println!("{} {} {} {} {}", a + b, c, d, o.0, o.1);
    let l: <[u8; 4] as Array>::Item = 5;
    let g = |x: <[u8; 4] as Array>::Item| x.wrapping_add(10);
    println!("{} {} {:?} {}", l.wrapping_add(255), g(250), keep::<[i16; 4]>(-3), plus::<[u32; 4]>(41));
}
