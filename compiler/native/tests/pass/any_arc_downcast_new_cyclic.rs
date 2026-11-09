// `Arc<dyn Any + Send + Sync>::downcast` and `Arc::new_cyclic`
// (TRIAGE T5): the owned downcast over the counted header, and
// a self-referential Arc whose Weak cannot upgrade during
// construction.
use std::any::Any;
use std::sync::{Arc, Weak};

struct Node {
    val: i32,
    me: Weak<Node>,
}

fn read<T>(v: T) -> T {
    v
}

fn main() {
    let v: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    println!("{} {:?}", v.is::<i32>(), v.downcast_ref::<i32>());
    match v.downcast::<i32>() {
        Ok(a) => println!("ok {}", *a),
        Err(_) => println!("err"),
    }
    let w: Arc<dyn Any + Send + Sync> = Arc::new(String::from("hi"));
    match w.downcast::<i32>() {
        Ok(a) => println!("ok {}", *a),
        Err(w2) => println!("err {:?}", w2.downcast_ref::<String>()),
    }
    let n: Arc<Node> = Arc::new_cyclic(|w: &Weak<Node>| {
        println!("inside {:?}", w.upgrade().map(|x| x.val));
        Node { val: read(7), me: w.clone() }
    });
    println!("{} {}", n.val, n.me.upgrade().map(|x| x.val).unwrap_or(-1));
    println!("{}", Arc::strong_count(&n));
}
