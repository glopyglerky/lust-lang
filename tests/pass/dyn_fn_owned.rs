// Owning `dyn Fn` values: `Rc<dyn Fn>` struct fields (the ts-rust
// visitor shape) with clones, `Box<dyn Fn>` returns and Vecs,
// `Box<dyn FnMut>` with state, and calls through `&*b`.
use std::rc::Rc;
struct Visitor {
    on_node: Rc<dyn Fn(i32) -> i32>,
}
fn walk(v: &Visitor) -> i32 {
    (v.on_node)(3)
}
fn make(add: i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |n| n + add)
}
fn apply(f: &dyn Fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}
fn main() {
    let v = Visitor { on_node: Rc::new(|n| n * 10) };
    println!("{}", walk(&v));
    let k = 7;
    let w = Visitor { on_node: Rc::new(move |n| n + k) };
    let w2 = Visitor { on_node: w.on_node.clone() };
    println!("{} {} {}", walk(&w), walk(&w2), Rc::strong_count(&w.on_node));
    let f = make(5);
    println!("{} {}", f(1), apply(&*f, 2));
    let fs: Vec<Box<dyn Fn(i32) -> i32>> = vec![make(1), make(2), Box::new(|n| n * n)];
    let total: i32 = fs.iter().map(|g| g(3)).sum();
    println!("{}", total);
    let mut count = 0;
    {
        let mut g: Box<dyn FnMut() -> i32> = Box::new(|| {
            count += 1;
            count
        });
        g();
        g();
        println!("{}", g());
    }
    println!("{}", count);
    let mut log: Vec<String> = Vec::new();
    let mut push: Box<dyn FnMut(&str)> = Box::new(move |s| {
        log.push(s.to_string());
        println!("{:?}", log);
    });
    push("a");
    push("b");
}
