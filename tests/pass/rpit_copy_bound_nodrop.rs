// An `impl Copy` value has no drop glue (rustc knows it from the bound),
// so holding one does not keep its captured borrow live to the end of
// the scope: each write to `c.n` below is accepted by rustc. The
// opaque still captures the borrow while it is used.
struct C {
    v: Vec<u32>,
    n: u8,
}
struct Wrap<I> {
    it: I,
}
fn mk(c: &C) -> impl Copy + std::fmt::Debug + '_ {
    &c.n
}
fn gmk<T: Copy>(t: T) -> impl Copy {
    t
}
fn go(c: &mut C) -> u8 {
    let r = &*c;
    let a = mk(r);
    let b = Some(gmk(&r.v));
    let w = Wrap { it: mk(r) };
    let t = (gmk(r), 1u8);
    let arr = [mk(r)];
    let f = move || {
        let _x = &a;
        1u8
    };
    println!("{:?} {}", a, b.is_some());
    c.n = 1;
    let r = &*c;
    let x = mk(r);
    println!("{:?}", x);
    c.n = 2;
    c.n
}
fn main() {
    let mut c = C { v: vec![1, 2, 3], n: 0 };
    println!("{}", go(&mut c));
}
