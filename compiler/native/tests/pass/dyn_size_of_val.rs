// ARCH 2.8: `size_of_val` and `align_of_val` on a dyn value read the size and
// align words of its vtable (through `&mut dyn`, upcasts, and `?Sized` generics).
use std::mem::{align_of_val, size_of_val};
trait Tr {
    fn f(&self) -> u32;
    fn bump(&mut self);
}
trait Sub: Tr {}
#[allow(dead_code)]
struct S { a: u64, b: u8 }
#[allow(dead_code)]
struct Big { x: [u16; 5], y: u8 }
impl Tr for S { fn f(&self) -> u32 { self.b as u32 } fn bump(&mut self) { self.b += 1; } }
impl Tr for u8 { fn f(&self) -> u32 { *self as u32 } fn bump(&mut self) { *self += 1; } }
impl Tr for Big { fn f(&self) -> u32 { self.y as u32 } fn bump(&mut self) { self.y += 2; } }
impl Tr for () { fn f(&self) -> u32 { 0 } fn bump(&mut self) {} }
impl Sub for Big {}
fn sz<T: ?Sized>(x: &T) -> (usize, usize) { (size_of_val(x), align_of_val(x)) }
fn main() {
    let s = S { a: 1, b: 2 };
    let d: &dyn Tr = &s;
    let e: &dyn Tr = &7u8;
    println!("{} {} {} {}", size_of_val(d), align_of_val(d), std::mem::size_of_val(e), core::mem::align_of_val(e));
    println!("{} {}", size_of_val(&d), align_of_val(&d));
    let mut b = Big { x: [1; 5], y: 3 };
    let m: &mut dyn Tr = &mut b;
    println!("{} {}", size_of_val(m), align_of_val(m));
    m.bump();
    println!("{} {:?}", m.f(), sz(m));
    let u: &dyn Tr = &();
    println!("{} {} {:?} {:?}", size_of_val(u), align_of_val(u), sz(u), sz(&s));
    let bb = Big { x: [0; 5], y: 9 };
    let sub: &dyn Sub = &bb;
    let up: &dyn Tr = sub;
    println!("{} {} {}", size_of_val(sub), size_of_val(up), align_of_val(&*up));
    println!("{} {}", align_of_val(&1u64), size_of_val(&[1u32; 3]));
    println!("{}", s.a + d.f() as u64 + e.f() as u64 + bb.x[0] as u64 + b.x[1] as u64);
}
