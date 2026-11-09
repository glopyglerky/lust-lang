// A panic inside a user `Index` impl reports the caller's `s[i]`: core's
// `Index::index` prototype is `#[track_caller]`, so every impl is (tf-panicloc).
use std::ops::{Index, IndexMut};
struct S(Vec<u8>);
impl Index<usize> for S {
    type Output = u8;
    fn index(&self, i: usize) -> &u8 {
        if i > 3 {
            panic!("big {}", i);
        }
        &self.0[i]
    }
}
impl IndexMut<usize> for S {
    fn index_mut(&mut self, i: usize) -> &mut u8 {
        &mut self.0[i]
    }
}
fn get(t: &S, i: usize) -> u8 {
    t[i]
}
fn main() {
    let mut s = S(vec![1, 2]);
    s[1] = 5;
    println!("{} {}", s[1], get(&s, 0));
    let n = std::env::args().count();
    if n > 5 {
        s[n] = 1;
    }
    println!("{}", get(&s, n + 4));
}
