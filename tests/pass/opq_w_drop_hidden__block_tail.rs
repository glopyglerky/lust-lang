// Probe w_drop_hidden__block_tail (probes2, w-opaque2): rustc and lust accept it with the same output.
#![allow(unused)]
struct C { v: Vec<u32>, n: u8 }
struct Wrap<I> { it: I }
struct DWrap<I> { it: I }
impl<I> Drop for DWrap<I> { fn drop(&mut self) { println!("dw"); } }
fn consume<T>(_t: T) {}
struct G<'a>(&'a u32);
impl Drop for G<'_> { fn drop(&mut self) {} }
fn mk(c: &C) -> impl Sized + '_ { G(&c.v[0]) }
fn go(c: &mut C, flag: bool) -> u8 {
    let r = &*c;
    let n = { let it = mk(r); 1u8 };
    c.n = n;
    c.n
}
fn main() {
    let mut c = C { v: vec![1, 2, 3], n: 0 };
    println!("{}", go(&mut c, false));
    println!("{}", go(&mut c, true));
}
