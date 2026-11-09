// Shared methods and fields through Rc's deref step.
use std::rc::Rc;
struct P { x: i32 }
impl P { fn get(&self) -> i32 { self.x * 3 } }
fn main() {
    let p = Rc::new(P { x: 7 });
    println!("{} {}", p.x, p.get());
    let v = Rc::new(vec![1, 2, 3]);
    println!("{} {}", v.len(), v[2]);
    let s = Rc::new("text".to_string());
    println!("{}", s.to_uppercase());
}
