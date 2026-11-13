// expect: L0312
// Probe w_gen_iter__array2 (probes2, w-opaque2): rustc and lust reject it.
#![allow(unused)]
struct C { v: Vec<u32>, n: u8 }
struct Wrap<I> { it: I }
struct DWrap<I> { it: I }
impl<I> Drop for DWrap<I> { fn drop(&mut self) { println!("dw"); } }
fn consume<T>(_t: T) {}
fn mk<T>(v: &[T]) -> impl Iterator<Item = &T> { v.iter() }
fn go(c: &mut C, flag: bool) -> u8 {
    let r = &*c;
    let it = [mk(&r.v), mk(&r.v)];
    c.n = 1;
    c.n
}
fn main() {
    let mut c = C { v: vec![1, 2, 3], n: 0 };
    println!("{}", go(&mut c, false));
    println!("{}", go(&mut c, true));
}
