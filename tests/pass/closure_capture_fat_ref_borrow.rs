// A closure that captures a fat reference (or a fat owner) by reference and
// borrows it again: `&s1` inside the closure is the env field itself (a
// `&&[u8]`), not one more deref of it. cgen typed the env field's place as
// unknown and took the sized `&[u8]` behind it for the unsized slice, so tcc
// rejected the C (`cannot convert struct Sl_sl_u8_ to struct Sl_sl_u8_ *`).
trait Tr {
    fn n(&self) -> usize;
}
impl Tr for u8 {
    fn n(&self) -> usize {
        *self as usize
    }
}
fn take<Q: ?Sized>(q: &Q) -> &Q {
    q
}
fn call<F: Fn() -> usize>(f: F) -> usize {
    f()
}
fn generic<T: std::fmt::Debug>(s: &[T]) -> usize {
    let f = || {
        let r = &s;
        r.len()
    };
    f() + call(|| take(&s).len())
}

fn main() {
    let s1: &[u8] = &[1];
    let f = || {
        let r = &s1;
        r.len()
    };
    println!("{}", f());

    let st: &str = "abc";
    let d: &dyn Tr = &7u8;
    let inner: &[u8] = &[9, 8];
    let rr: &&[u8] = &inner;
    let b: Box<[u16]> = vec![4, 5, 6].into_boxed_slice();
    let bs: Box<&[u8]> = Box::new(&[3, 3, 3, 3]);
    let g = || {
        let a = &st;
        let e = &d;
        let x = &rr;
        let y = &b;
        let z = &*bs;
        let h = || &st;
        (a.len() + h().len(), e.n(), x[1], y[2], z.len(), take(&d).n())
    };
    println!("{:?} {:?}", g(), g());

    let mut arr = [5u8, 6];
    let mut m: &mut [u8] = &mut arr;
    let mut k = || {
        let r = &mut m;
        r[0] += 1;
        r.len() * 100 + r[0] as usize
    };
    println!("{} {}", k(), k());
    println!("{:?} {}", m, generic(&["x", "y"]));
}
