// Probe g_gen_into__block_scope (probes, w-opaque2): rustc and lust accept it with the same output.
#![allow(unused)]
struct C { v: Vec<u32>, n: u8, g: G<u32> }
struct G<K> { e: Vec<K> }
struct Wrap<I> { it: I }
struct Loud(u32);
impl Drop for Loud { fn drop(&mut self) { println!("drop {}", self.0); } }
fn mk<I: IntoIterator>(i: I) -> impl Iterator<Item = I::Item> { i.into_iter() }
fn go(c: &mut C, flag: bool) -> u8 {
    let r = &*c;
    
    { let mut it = mk(&r.v); it.next(); }
    c.n = 1;
    c.n
}
fn main() {
    let mut c = C { v: vec![1, 2, 3], n: 0, g: G { e: vec![4, 5] } };
    println!("{}", go(&mut c, false));
    println!("{}", go(&mut c, true));
}
