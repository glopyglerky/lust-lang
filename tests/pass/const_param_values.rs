// Const generic params as values and repeat lengths in generic
// bodies (fns, impls, closures), and const turbofish args: negative,
// bool, char, braced, forwarded, and on the type segment.
fn record<const N: usize>(b: &[u8]) -> [u8; N] {
    let mut out = [0u8; N];
    let mut i = 0;
    while i < N && i < b.len() {
        out[i] = b[i];
        i += 1;
    }
    out
}
fn flag<const B: bool>() -> &'static str {
    if B { "yes" } else { "no" }
}
fn sym<const C: char>() -> char {
    C
}
fn neg<const K: i32>() -> i32 {
    K * 2
}
struct Ring<const N: usize> {
    d: [u16; N],
}
impl<const N: usize> Ring<N> {
    fn new() -> Self {
        Ring { d: [1; N] }
    }
    fn cap(&self) -> usize {
        N + self.d.len()
    }
}
fn twice<T: Copy + std::fmt::Debug, const N: usize>(x: T) -> Vec<[T; N]> {
    let f = |y: T| [y; N];
    vec![f(x), [x; N]]
}
fn main() {
    let a: [u8; 3] = record(b"hello");
    let b = record::<6>(b"hi");
    println!("{:?} {:?}", a, b);
    println!("{} {} {} {}", flag::<true>(), flag::<false>(), sym::<'z'>(), neg::<{ -3 }>());
    let r = Ring::<4>::new();
    println!("{} {:?}", r.cap(), r.d);
    println!("{:?}", twice::<u8, 2>(5));
    more();
}
struct Buf<const N: usize> {
    v: [u8; N],
}
impl<const N: usize> Buf<N> {
    fn new() -> Self {
        Buf { v: [2; N] }
    }
}
fn len<const N: usize>() -> usize {
    N
}
fn fwd<const N: usize>() -> usize {
    len::<N>() + len::<{ N }>()
}
fn more() {
    let b = Buf::<5>::new();
    let c = Buf::<{ 2 + 1 }>::new();
    println!("{:?} {:?} {}", b.v, c.v, fwd::<4>());
}
