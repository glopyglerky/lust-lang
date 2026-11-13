// The derive delta (sigedit P3.2): toggles on A reach W and Outer (field
// users), a const of A, and a fn bound; Eq and Hash go and come back
// (dead records revive), Debug and Copy go while a field user needs them.
const _ROUTE: u8 = 0;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct A {
    x: u8,
}
#[derive(Clone, Copy, Debug)]
struct W {
    a: A,
    n: u8,
}
#[derive(Clone, Debug)]
struct Outer {
    w: W,
}
const CA: A = A { x: 3 };
fn twice<T: Copy>(t: T) -> (T, T) {
    (t, t)
}
fn main() {
    let w = W { a: CA, n: 1 };
    let (p, q) = twice(w);
    let o = Outer { w: p };
    let c = o.clone();
    println!("{:?} {} {}", c, q.n, CA == A { x: 3 });
}
