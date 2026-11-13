// A dead fn's signature names `Own<dyn P + 'static>`, which mono never materializes, next to the
// body-inferred `Own<dyn P>` instance: both must be one C type (tcc: cannot convert struct).
#![allow(dead_code)]
use std::rc::Rc;
trait P { fn id(&self) -> u32; }
struct A(u32);
impl P for A { fn id(&self) -> u32 { self.0 } }
struct Own<T: ?Sized>(Box<T>);
enum E<T> { V(T), N }
struct Pair<X, Y> { a: X, b: Y }
fn f1(x: Own<dyn P>) -> Own<dyn P + 'static> { let y: Own<dyn P + 'static> = Own(x.0); y }
fn f2(x: Box<dyn P>) -> E<Box<dyn P + 'static>> { let e: E<Box<dyn P>> = E::V(x); e }
fn f3(x: Box<dyn P>) -> Pair<Rc<dyn P + 'static>, u32> { let p: Pair<Rc<dyn P>, u32> = Pair { a: Rc::from(x), b: 1 }; p }
fn f4(x: Own<dyn P>) -> Vec<Own<dyn P + 'static>> { vec![x] }
fn f5(x: E<Box<dyn P + 'static>>) -> E<Box<dyn P>> { match x { E::V(b) => E::V(b), E::N => E::N } }
fn dead(x: Own<dyn P>) -> Own<dyn P + 'static> { x }
fn main() {
    let o = f1(Own(Box::new(A(2))));
    let k = match f5(E::V(Box::new(A(6)))) { E::V(b) => b.id(), E::N => 0 };
    println!("{} {}", o.0.id(), k);
}
