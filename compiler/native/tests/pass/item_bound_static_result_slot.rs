// A callee with an `Item = X` bound whose result has a `'static` slot (`Result<_, &'static str>`):
// the over-approximation that lets argument regions reach the result skips that slot
// (ts-rust effect/ls/schemagen.rs process_node).
#[derive(Clone, Copy, Debug)]
pub struct Node(u32);
#[derive(Clone, Copy, Debug)]
pub struct NodeSlice(&'static [Node]);
pub struct NodeSliceIter { s: &'static [Node], i: usize }
impl Iterator for NodeSliceIter {
    type Item = Node;
    fn next(&mut self) -> Option<Node> { let n = self.s.get(self.i).copied(); self.i += 1; n }
}
impl IntoIterator for NodeSlice {
    type Item = Node;
    type IntoIter = NodeSliceIter;
    fn into_iter(self) -> NodeSliceIter { NodeSliceIter { s: self.0, i: 0 } }
}
impl IntoIterator for &NodeSlice {
    type Item = Node;
    type IntoIter = NodeSliceIter;
    fn into_iter(self) -> NodeSliceIter { NodeSliceIter { s: self.0, i: 0 } }
}
struct NodeFactory { n: u32 }
struct SchemaGen<'f> { f: &'f NodeFactory }
static NODES: [Node; 2] = [Node(1), Node(2)];
impl<'f> SchemaGen<'f> {
    fn process_node(&self, node: Node) -> Result<u32, &'static str> {
        if node.0 > 100 { return Err("big"); }
        if node.0 == 0 {
            let types = NodeSlice(&NODES);
            let members = self.process_node_list(types)?;
            return Ok(members.len() as u32 + self.f.n);
        }
        Ok(node.0)
    }
    fn process_node_list(&self, nodes: impl IntoIterator<Item = Node>) -> Result<Vec<u32>, &'static str> {
        nodes.into_iter().map(|n| self.process_node(n)).collect()
    }
}
fn main() {
    let f = NodeFactory { n: 7 };
    let g = SchemaGen { f: &f };
    println!("{:?}", g.process_node(Node(0)));
}
