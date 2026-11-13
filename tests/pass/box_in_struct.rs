// Option<Box<Node>> as a struct field.
#[derive(Debug)]
struct Node { v: i32, next: Option<Box<Node>> }
fn count(n: &Node) -> i32 {
    match &n.next { Some(next) => 1 + count(next), None => 1 }
}
fn main() {
    let n = Node { v: 1, next: Some(Box::new(Node { v: 2, next: None })) };
    println!("{} {:?}", count(&n), n);
}
