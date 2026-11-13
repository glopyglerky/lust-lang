// expect: L0733
// rustc: lifetime may not live long enough. The closure's output `Option<Sender<Box<dyn P>>>`
// is invariant in the object lifetime, so the return type's `'static` reaches the boxed borrow.
use std::sync::mpsc::Sender;
trait P { fn p(&self) -> u32; }
struct B<'b>(&'b u32);
impl<'b> P for B<'b> { fn p(&self) -> u32 { *self.0 } }
fn mk(n: &u32) -> Option<Sender<Box<dyn P>>> {
    let f = move || {
        let (send, _receive) = std::sync::mpsc::channel::<Box<dyn P>>();
        send.send(Box::new(B(n))).ok();
        Some(send)
    };
    f()
}
fn main() { println!("{}", mk(&3).is_some()); }
