// expect: L0733
// rustc: lifetime may not live long enough. `MutexGuard<'b, dyn P>` is `MutexGuard<'b, dyn P + 'b>`.
use std::sync::{Mutex, MutexGuard};
trait P { fn id(&self) -> u32; }
struct A(u32);
impl P for A { fn id(&self) -> u32 { self.0 } }
fn f<'b>(r: MutexGuard<'b, dyn P>) -> u32 {
    let x: &(dyn P + 'static) = &*r;
    x.id()
}
fn main() {
    let m: Box<Mutex<dyn P>> = Box::new(Mutex::new(A(3)));
    println!("{}", f(m.lock().unwrap()));
}
