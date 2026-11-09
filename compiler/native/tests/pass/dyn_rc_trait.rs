// `Rc<dyn Tr>`: clones share the value (strong counts), struct fields,
// and the value drops with the last clone.
use std::rc::Rc;
trait Node { fn val(&self) -> i32; }
struct Leaf(i32);
struct D(&'static str);
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); } }
impl Node for Leaf { fn val(&self) -> i32 { self.0 } }
impl Node for D { fn val(&self) -> i32 { self.0.len() as i32 } }
struct Holder { n: Rc<dyn Node> }
fn main() {
    let a: Rc<dyn Node> = Rc::new(Leaf(7));
    let b = a.clone();
    println!("{} {} {}", a.val(), b.val(), Rc::strong_count(&a));
    let h = Holder { n: Rc::new(D("held")) };
    let h2 = Holder { n: h.n.clone() };
    drop(h);
    println!("still {}", h2.n.val());
    drop(h2);
    println!("end");
}
