// `dyn Fn` edits on one warm daemon: the closure body behind the vtable,
// its captures (the env layout and the vtable size word), the `dyn Fn`
// signature at both ends, an FnMut body where `dyn Fn` is wanted (an
// error), and a `Box<dyn FnOnce>` that consumes its env.
use std::rc::Rc;
struct Visitor {
    on_node: Rc<dyn Fn(i32) -> i32>,
}
fn walk(v: &Visitor, n: i32) -> i32 {
    (v.on_node)(n)
}
fn call(f: &dyn Fn(i32) -> i32) -> i32 {
    f(3) + f(4)
}
fn run(f: Box<dyn FnOnce() -> String>) -> String {
    f()
}
fn main() {
    let k = 5;
    let add = |n: i32| n + k;
    println!("{}", call(&add));
    let v = Visitor { on_node: Rc::new(|n| n * 10) };
    println!("{}", walk(&v, 2));
    let s = String::from("text");
    println!("{}", run(Box::new(move || s + "!")));
}
