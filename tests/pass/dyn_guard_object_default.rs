// A guard over `dyn P` defaults its object lifetime to the guard's lifetime, so a `'static`
// object needs a written `+ 'static`; a `RefCell<dyn P>` behind a reference defaults to
// `'static` (no bound on its param).
use std::cell::RefCell;
use std::sync::{Mutex, MutexGuard};
trait P { fn id(&self) -> u32; }
struct A(u32);
impl P for A { fn id(&self) -> u32 { self.0 } }
fn long<'b>(r: MutexGuard<'b, dyn P + 'static>) -> u32 { let x: &(dyn P + 'static) = &*r; x.id() }
fn cell<'b>(r: &'b RefCell<dyn P>) -> u32 { let g = r.borrow(); let x: &(dyn P + 'static) = &*g; x.id() }
fn main() {
    let m: Box<Mutex<dyn P>> = Box::new(Mutex::new(A(3)));
    let b = long(m.lock().unwrap());
    let c: Box<RefCell<dyn P>> = Box::new(RefCell::new(A(4)));
    println!("{} {}", b, cell(&c));
}
