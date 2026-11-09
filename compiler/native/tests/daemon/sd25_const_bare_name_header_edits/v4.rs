// Bare names `end` in reached const fn bodies (a field, a param, a binding, struct literal
// shorthand) while Ty::end's header is edited: they cannot name the method (round 5).
#![allow(dead_code, non_upper_case_globals, unused_imports)]
use std::sync::LazyLock;
struct Ty { x: u32 }
impl Ty {
    fn end(&self, x: u32) -> u64 { (self.x + x) as u64
    }
}
struct Bits(u32);
impl Bits {
    const fn end(&self, x: u32) -> u32 { self.0 + x }
}
struct P14 { end: u32 }
const fn cf14(p: P14) -> u32 { let end = p.end; end + 1 }
const C14: u32 = cf14(P14 { end: 2 });
const fn cf15(end: u32) -> u32 { end * 2 }
const C15: u32 = cf15(4);
struct Q17 { end: u32 }
const fn cf17(end: u32) -> Q17 { Q17 { end } }
const C17: u32 = cf17(5).end;
fn wrap<T: Copy>(t: T) -> T { t }
fn call() -> u32 { Ty { x: 1 }.end(1) as u32 }
fn main() {
    println!("{} {}", C14 + C15 + C17, wrap(call()));
}
