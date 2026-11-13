// Tuple-struct fields through user Deref and DerefMut impls (ManuallyDrop and
// a user wrapper): reads, writes, borrows, closure captures.
use std::mem::ManuallyDrop;
struct W<T> { v: T }
impl<T> std::ops::Deref for W<T> { type Target = T; fn deref(&self) -> &T { &self.v } }
struct N(&'static str, i32);
struct D<'a>(&'a i32);
impl Drop for D<'_> {
    fn drop(&mut self) { println!("drop {}", self.0); }
}
fn main() {
    let a = W { v: N("x", 1) };
    println!("{} {}", a.0, a.1);
    let mut m = ManuallyDrop::new(N("y", 2));
    m.1 += 5;
    m.0 = "z";
    println!("{} {}", m.0, m.1);
    let r = &mut m.1;
    *r = 9;
    println!("{}", m.1);
    let t = ManuallyDrop::new((1, String::from("s")));
    println!("{} {}", t.0, t.1);
    drop(ManuallyDrop::into_inner(t));
    let c = || println!("cl {}", m.0);
    c();
    let mut k = ManuallyDrop::new(N("k", 0));
    let mut inc = || k.1 += 1;
    inc();
    inc();
    println!("{}", k.1);
    let dm;
    {
        let x = 1;
        dm = ManuallyDrop::new(D(&x));
        println!("{}", dm.0);
    }
    let ww = W { v: ManuallyDrop::new(N("deep", 3)) };
    println!("{}", ww.1);
    println!("end");
}
