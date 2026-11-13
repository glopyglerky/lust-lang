// `Cell::new` in const items, const blocks, array repeats and const fns
// (ts-rust core.rs new_cache: `[const { Cell::new((0, 0, "")) }; WAYS]`).
// Each use of the const is a fresh value.
use std::cell::Cell;
const C: Cell<u64> = Cell::new(1 << 40);
struct Set([Cell<(u32, u32, &'static str)>; 2]);
const fn mk(v: i16) -> Cell<i16> { Cell::new(v) }
const M: Cell<i16> = mk(-7);
fn main() {
    const EMPTY: Set = Set([const { Cell::new((0, 0, "")) }; 2]);
    let v: Vec<Set> = (0..3).map(|_| EMPTY).collect();
    v[1].0[0].set((1, 2, "a"));
    println!("{:?} {:?} {:?}", v[1].0[0].get(), v[0].0[0].get(), EMPTY.0[0].get());
    C.set(5);
    let r = &C;
    r.set(6);
    println!("{} {} {}", C.get(), r.get(), M.get());
    let boxed: Box<[Set]> = (0..2).map(|_| EMPTY).collect();
    println!("{:?}", boxed[1].0[1].get());
}
