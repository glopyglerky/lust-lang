// A generic impl over an integer literal: `Wrap<{integer}>: Show`
// selects the impl but waits in the queue until the literal is typed.
// Each queue run selects it once (it used to repeat every round, and
// this file took 3.5 s to check). Nested wraps and calls between the
// run points must still pick the right impls.
trait Show {
    fn show(&self) -> u32;
}
impl Show for u8 {
    fn show(&self) -> u32 {
        *self as u32 + 100
    }
}
impl Show for u16 {
    fn show(&self) -> u32 {
        *self as u32 + 200
    }
}
impl Show for i32 {
    fn show(&self) -> u32 {
        *self as u32
    }
}
struct Wrap<T>(T);
impl<T: Show> Show for Wrap<T> {
    fn show(&self) -> u32 {
        self.0.show() + 1
    }
}
struct Num(u32);
impl Num {
    fn get(&self) -> u32 {
        self.0
    }
}
fn need<S: Show>(s: S) -> u32 {
    s.show()
}
fn main() {
    let a = need(Wrap(5));
    let n = Num(a);
    let b = n.get() + n.get();
    let c = n.get() * 2;
    let d = need(Wrap(Wrap(7)));
    let e = Num(d).get();
    let f = need(Wrap(3u8)) + need(Wrap(Wrap(4u16)));
    let x = 9;
    let g = need(Wrap(x));
    let y: u16 = x;
    println!("{} {} {} {} {} {} {}", a, b, c, e, f, g, y);
}
