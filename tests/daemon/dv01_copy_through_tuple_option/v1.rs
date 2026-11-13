#[derive(Clone, Copy, Debug)]
struct A {
    x: u8,
}
fn twice<T: Copy>(t: T) -> (T, T) {
    (t, t)
}
fn pair() -> (A, u8) {
    (A { x: 1 }, 2)
}
fn main() {
    let t = pair();
    let u = t;
    let o = Some(A { x: 3 });
    let p = o;
    let w = twice(o);
    println!("{} {} {:?} {:?} {:?}", t.0.x, u.1, o, p, w.0);
}
