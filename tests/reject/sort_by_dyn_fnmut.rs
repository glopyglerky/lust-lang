// expect: L0702
// Valid Rust: a `&mut dyn FnMut` comparator. lust's sort adapter calls closures, fn items
// and fn pointers only, so it rejects a dyn one at check (not a codegen panic).
use std::cmp::Ordering;

fn main() {
    let mut v = vec![3u32, 1, 2];
    let mut g = |a: &u32, b: &u32| a.cmp(b);
    let d: &mut dyn FnMut(&u32, &u32) -> Ordering = &mut g;
    v.sort_by(d);
    println!("{v:?}");
}
