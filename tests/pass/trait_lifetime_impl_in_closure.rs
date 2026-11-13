// ts-rust node.rs for_each_scoped_child: inside a closure, `impl<'d> ChildVisit<'d>
// for ScopedChildVisit` holds for the closure argument's lifetime (no 'static).
#[derive(Clone, Copy)]
pub struct Node(u32);
pub struct NodeData { kids: Vec<u32> }
pub type ListHook<'a> = &'a mut dyn FnMut(u32, bool);
trait ChildVisit<'d> { fn node(&mut self, id: u32) -> bool; fn list(&mut self, l: &'d Vec<u32>) -> bool; }
struct ScopedChildVisit<'a, 'b> { n: Node, file: usize, v: &'a mut dyn FnMut(Node) -> bool, lists: Option<ListHook<'b>> }
impl<'d> ChildVisit<'d> for ScopedChildVisit<'_, '_> {
    fn node(&mut self, id: u32) -> bool { (self.v)(Node(id + self.n.0 + self.file as u32)) }
    fn list(&mut self, l: &'d Vec<u32>) -> bool { if let Some(h) = self.lists.as_mut() { h(l.len() as u32, true); } l.iter().any(|&i| self.node(i)) }
}
fn walk_children<'d>(data: &'d NodeData, w: &mut impl ChildVisit<'d>) -> bool { w.list(&data.kids) }
pub fn with_scoped_ast_node<R>(n: Node, f: impl FnOnce(&NodeData) -> R) -> R { let d = NodeData { kids: vec![n.0, 2] }; f(&d) }
fn for_each_scoped_child(n: Node, v: &mut dyn FnMut(Node) -> bool, lists: Option<ListHook>) -> bool {
    with_scoped_ast_node(n, |data| walk_children(data, &mut ScopedChildVisit { n, file: 0, v, lists }))
}
fn main() { let mut c = 0; let mut f = |x: Node| { c += x.0; false }; println!("{}", for_each_scoped_child(Node(1), &mut f, None)); println!("{}", c); }
