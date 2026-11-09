// `MaybeUninit::uninit` as a value (ts-rust leak_arena.rs `alloc_with`:
// `try_alloc_with(MaybeUninit::<T>::uninit)`), with and without the type
// turbofish, in a generic fn, as a fn pointer, and for `resize_with`. Only
// values without heap parts: a `MaybeUninit` never drops what it holds.
use std::mem::MaybeUninit;
fn mk<T>(f: impl FnOnce() -> MaybeUninit<T>) -> MaybeUninit<T> {
    f()
}
fn g<T: std::fmt::Debug>(x: T) {
    let mut m = mk(MaybeUninit::<T>::uninit);
    println!("{:?}", m.write(x));
}
fn main() {
    let mut a = mk(MaybeUninit::<u32>::uninit);
    let mut b: MaybeUninit<(u8, u64)> = mk(MaybeUninit::zeroed);
    println!("{} {:?}", a.write(5), b.write((1, 2)));
    g(7u16);
    g((1u8, 'c'));
    let f: fn() -> MaybeUninit<u8> = MaybeUninit::uninit;
    let mut c = f();
    println!("{}", c.write(3));
    let mut v: Vec<MaybeUninit<u32>> = Vec::new();
    v.resize_with(3, MaybeUninit::uninit);
    println!("{}", v.len());
}
