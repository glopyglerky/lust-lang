use std::cell::RefCell; use std::rc::Rc;
trait Node { fn add(&mut self, x: i64); fn total(&self) -> i64; }
struct Sum(i64); struct Max(i64);
impl Node for Sum { fn add(&mut self, x: i64) { self.0 += x; } fn total(&self) -> i64 { self.0 } }
impl Node for Max { fn add(&mut self, x: i64) { if x > self.0 { self.0 = x; } } fn total(&self) -> i64 { self.0 } }
fn mk(max: bool) -> Rc<RefCell<dyn Node>> { if max { Rc::new(RefCell::new(Max(i64::MIN))) } else { Rc::new(RefCell::new(Sum(0))) } }
fn main() {
    let nodes: Vec<Rc<RefCell<dyn Node>>> = vec![mk(false), mk(true), mk(false)];
    for (i, n) in nodes.iter().enumerate() { for x in [3, -1, 7] { n.borrow_mut().add(x * i as i64); } }
    let t: Vec<i64> = nodes.iter().map(|n| n.borrow().total()).collect();
    println!("{:?}", t);
}
