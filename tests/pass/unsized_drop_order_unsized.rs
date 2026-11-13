use std::rc::Rc;
struct N(u32);
impl Drop for N { fn drop(&mut self) { println!("drop {}", self.0); } }
fn main() {
    let a: Rc<[N]> = Rc::from(vec![N(1), N(2)]);
    let b = a.clone();
    drop(a);
    println!("after a");
    drop(b);
    println!("after b");
    let c: Rc<[N]> = Rc::new([N(3), N(4)]);
    let _d = c.clone();
    println!("end");
}
