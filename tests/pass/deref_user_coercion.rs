// Deref coercions through user Deref impls: `&name` to `&str`, a
// reference to a reference, `&mut w` to `&mut P` through `DerefMut`, a
// chain of two impls, and a Vec of coerced elements.
use std::ops::{Deref, DerefMut};
struct Name(String);
impl Deref for Name { type Target = str; fn deref(&self) -> &str { println!("deref"); &self.0 } }
struct P { x: i32 }
struct W { p: P }
impl Deref for W { type Target = P; fn deref(&self) -> &P { &self.p } }
impl DerefMut for W { fn deref_mut(&mut self) -> &mut P { println!("deref_mut"); &mut self.p } }
struct Outer { w: W }
impl Deref for Outer { type Target = W; fn deref(&self) -> &W { &self.w } }
fn takes_str(s: &str) -> usize { s.len() }
fn takes_p(p: &P) -> i32 { p.x }
fn bump(p: &mut P) { p.x += 1; }
fn main() {
    let n = Name(String::from("hello"));
    println!("{}", takes_str(&n));
    let r = &n;
    println!("{}", takes_str(r));
    let s: &str = &n;
    println!("{}", s);
    let mut w = W { p: P { x: 1 } };
    println!("{}", takes_p(&w));
    bump(&mut w);
    println!("{}", w.p.x);
    let o = Outer { w: W { p: P { x: 7 } } };
    println!("{}", takes_p(&o));
    let v: Vec<&str> = vec![&n, "x"];
    println!("{:?}", v);
}
