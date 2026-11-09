// ts-rust ast/visitor.rs (R11): `Rc<dyn Fn(Node, &mut NodeVisitor<'a, C>) -> Node + 'a>` with
// `C = &mut Counter`, made from an `impl Fn(..) + 'a` argument.
use std::rc::Rc;
#[derive(Clone, Copy, Debug, PartialEq)]
struct Node(u32);
struct NodeFactory { base: u32 }
type VisitFn<'a, C> = Rc<dyn Fn(Node, &mut NodeVisitor<'a, C>) -> Node + 'a>;
struct NodeVisitor<'a, C = ()> {
    visit: Option<VisitFn<'a, C>>,
    factory: Option<&'a NodeFactory>,
    ctx: C,
}
fn new_node_visitor<'a, C>(visit: impl Fn(Node, &mut NodeVisitor<'a, C>) -> Node + 'a, factory: Option<&'a NodeFactory>, ctx: C) -> NodeVisitor<'a, C> {
    NodeVisitor { visit: Some(Rc::new(visit)), factory, ctx }
}
impl<'a, C> NodeVisitor<'a, C> {
    fn call_visit(&mut self, node: Node) -> Node {
        let visit = self.visit.clone().expect("nil");
        visit(node, self)
    }
    fn visit_node(&mut self, node: Node) -> Node {
        if self.visit.is_none() { return node; }
        self.call_visit(node)
    }
}
struct Counter { n: u32 }
fn main() {
    let f = NodeFactory { base: 100 };
    let mut c = Counter { n: 0 };
    {
        let mut v = new_node_visitor(|n: Node, v: &mut NodeVisitor<'_, &mut Counter>| {
            v.ctx.n += 1;
            Node(n.0 + v.factory.map_or(0, |f| f.base))
        }, Some(&f), &mut c);
        let a = v.visit_node(Node(1));
        let b = v.visit_node(a);
        println!("{:?} {:?}", a, b);
    }
    println!("{}", c.n);
}
