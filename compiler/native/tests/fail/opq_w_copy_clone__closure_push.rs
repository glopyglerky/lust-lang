// expect: L0312
// Probe w_copy_clone__closure_push (probes2, w-opaque2): rustc and lust reject it.
#![allow(unused)]
struct C { v: Vec<u32>, n: u8 }
struct Wrap<I> { it: I }
struct DWrap<I> { it: I }
impl<I> Drop for DWrap<I> { fn drop(&mut self) { println!("dw"); } }
fn consume<T>(_t: T) {}
fn mk(c: &C) -> impl Clone + Copy + std::fmt::Debug + '_ { (&c.v, 1u8) }
fn go(c: &mut C, flag: bool) -> u8 {
    let r = &*c;
    let mut v = Vec::new();
    let mut f = |x| v.push(x);
    f(mk(r));
    c.n = 1;
    c.n
}
fn main() {
    let mut c = C { v: vec![1, 2, 3], n: 0 };
    println!("{}", go(&mut c, false));
    println!("{}", go(&mut c, true));
}
