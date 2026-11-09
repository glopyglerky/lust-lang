// expect: L0730
// rustc E0320: G needs no drop, but the temporary of `G { .. }` is still drop-checked.
use std::marker::PhantomData;
struct G<T> { v: T, p: PhantomData<G<Option<T>>> }
fn set(p: &mut G<u8>) { *p = G { v: 7, p: PhantomData }; }
fn main() { println!("ok"); }
