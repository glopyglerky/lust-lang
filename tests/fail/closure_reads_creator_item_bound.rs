// expect: L0306
// rustc E0597: in the closure, `v.extend(it)` makes `v`'s items `&'a u8` exactly (the enclosing fn's
// bound), so `&x0` (the closure's own copy) must live for `'a` (tf-borrow p51).
use std::collections::{VecDeque, BTreeSet, HashSet};
fn g<'a, 'b>(it: impl Iterator<Item = &'a u8>, x0: u8, ra: &'a u8, rb: &'b u8) -> usize {
    let f = move || {
        let mut v = Vec::new();
        v.extend(it);
        if v.len() > 50 { return v.len(); }
        v.push(&x0);
        v.len()
    }; f()
}
fn main() { let s = vec![1u8, 2, 3]; let a0 = 7u8; let b0 = 8u8; println!("{}", g(s.iter(), 9u8, &a0, &b0)); }
