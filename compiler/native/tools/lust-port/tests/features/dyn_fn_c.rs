// feature: dyn.fn
// form: stored callback as Rc<dyn Fn> in a struct field, as in ts-rust visitors
use std::rc::Rc;
struct Visitor { on_node: Rc<dyn Fn(i32) -> i32> }
fn walk(v: &Visitor) -> i32 { (v.on_node)(3) }
fn main() {
    let v = Visitor { on_node: Rc::new(|n| n * 10) };
    println!("{}", walk(&v));
}
