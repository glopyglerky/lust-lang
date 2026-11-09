// A parent/child tree with Weak parent links (the usual Rc<RefCell<..>>
// shape): upgrade while alive, drop order when the root goes.
use std::cell::RefCell;
use std::rc::{Rc, Weak};
struct Node {
    name: String,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}
impl Drop for Node {
    fn drop(&mut self) {
        println!("drop {}", self.name);
    }
}
fn node(name: &str) -> Rc<Node> {
    Rc::new(Node { name: name.to_string(), parent: RefCell::new(Weak::new()), children: RefCell::new(Vec::new()) })
}
fn main() {
    let root = node("root");
    for n in ["a", "b"] {
        let child = node(n);
        *child.parent.borrow_mut() = Rc::downgrade(&root);
        root.children.borrow_mut().push(child);
    }
    let first = Rc::clone(&root.children.borrow()[0]);
    match first.parent.borrow().upgrade() {
        Some(p) => println!("{} -> {}", first.name, p.name),
        None => println!("orphan"),
    }
    println!("strong {} weak {}", Rc::strong_count(&root), Rc::weak_count(&root));
    drop(root);
    println!("root gone: {}", first.parent.borrow().upgrade().is_none());
}
