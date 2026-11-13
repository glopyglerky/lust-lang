// Rc<RefCell<T>> fixture set: a tree with Weak parent links, paths through upgrade, counts, and drop order.
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
struct Node {
    name: String,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

impl Node {
    fn new(name: &str) -> Rc<Node> {
        Rc::new(Node { name: name.to_string(), parent: RefCell::new(Weak::new()), children: RefCell::new(Vec::new()) })
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        println!("drop {}", self.name);
    }
}

fn add_child(parent: &Rc<Node>, child: Rc<Node>) {
    *child.parent.borrow_mut() = Rc::downgrade(parent);
    parent.children.borrow_mut().push(child);
}

fn path(n: &Rc<Node>) -> String {
    match n.parent.borrow().upgrade() {
        Some(p) => format!("{}/{}", path(&p), n.name),
        None => n.name.clone(),
    }
}

fn main() {
    let root = Node::new("root");
    let a = Node::new("a");
    let b = Node::new("b");
    add_child(&root, a.clone());
    add_child(&a, b.clone());
    println!("{}", path(&b));
    println!("{} {} {}", Rc::strong_count(&a), Rc::weak_count(&a), Rc::weak_count(&root));
    let names: Vec<String> = root.children.borrow().iter().map(|c| c.name.clone()).collect();
    println!("{:?}", names);
    drop(b);
    drop(a);
    println!("before root");
    drop(root);
    println!("after root");
}
