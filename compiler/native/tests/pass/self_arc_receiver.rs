// `self: &Arc<Self>` and `self: Arc<Self>` receivers (ts-rust lsp/server.rs read_loop).
use std::rc::Rc;
use std::sync::Arc;
struct S { x: i32 }
impl S {
    fn get(self: &Arc<Self>) -> i32 { self.x }
    fn take(self: Arc<Self>) -> i32 { self.x + 1 }
    fn boxed(self: &Box<Rc<Self>>) -> i32 { self.x + 2 }
    fn full(self: &std::sync::Arc<S>) -> i32 { self.x + 3 }
}
trait G { fn g(self: Arc<Self>) -> i32; }
impl G for S { fn g(self: Arc<Self>) -> i32 { self.x * 10 } }
fn main() {
    let a = Arc::new(S { x: 4 });
    println!("{} {} {}", a.get(), a.full(), Arc::clone(&a).take());
    println!("{}", a.clone().g());
    println!("{}", Box::new(Rc::new(S { x: 1 })).boxed());
}
