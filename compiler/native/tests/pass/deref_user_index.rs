// Indexing through user Deref impls: a slice wrapper and a Vec wrapper,
// reads (`deref`), writes and compound assignment (`deref_mut`), a range
// index, a borrow of an element, and a closure that captures the base.
use std::ops::{Deref, DerefMut};
struct Sv { items: Vec<u32> }
impl Deref for Sv { type Target = [u32]; fn deref(&self) -> &[u32] { println!("deref"); &self.items } }
impl DerefMut for Sv { fn deref_mut(&mut self) -> &mut [u32] { println!("deref_mut"); &mut self.items } }
struct Wrap { v: Vec<String> }
impl Deref for Wrap { type Target = Vec<String>; fn deref(&self) -> &Vec<String> { &self.v } }
fn main() {
    let mut s = Sv { items: vec![10, 20, 30] };
    println!("{}", s[1]);
    s[0] = 5;
    s[2] += 1;
    println!("{:?} {:?}", &s[..2], s.items);
    let w = Wrap { v: vec![String::from("a"), String::from("b")] };
    println!("{} {}", w[1], w[0].len());
    let r = &w[0];
    println!("{}", r);
    let get = || s[1] + 1;
    println!("{}", get());
}
