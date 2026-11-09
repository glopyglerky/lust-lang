// macro: n, o, l, ol, m, p, po, pl, pol, pm, el, e
// source: crates/ts_goport/src/ast/node.rs:4157
// source: crates/ts_goport/src/ast/node.rs:4162
// source: crates/ts_goport/src/ast/node.rs:4167
// source: crates/ts_goport/src/ast/node.rs:4172
// source: crates/ts_goport/src/ast/node.rs:4177
// source: crates/ts_goport/src/ast/node.rs:4886
// source: crates/ts_goport/src/ast/node.rs:4891
// source: crates/ts_goport/src/ast/node.rs:4896
// source: crates/ts_goport/src/ast/node.rs:4901
// source: crates/ts_goport/src/ast/node.rs:4906
// source: crates/ts_goport/src/ast/node.rs:4911
// source: crates/ts_goport/src/ast/node.rs:4916
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
include!("node_scaffold.rs.inc");

use crate::astdata::NodeId;

// The walker stand-in for ChildVisit: record what the fn-local
// shorthand macros visit. Returning true stops the walk, as in Go.
pub struct Recorder { pub seen: Vec<String> }
impl Recorder {
    pub fn node(&mut self, id: NodeId) -> bool { self.seen.push(format!("n{}", id.0)); false }
    pub fn opt(&mut self, id: Option<NodeId>) -> bool { self.seen.push(format!("o{}", id.map_or(0, |i| i.0))); false }
    pub fn list(&mut self, l: &crate::astdata::NodeList) -> bool { self.seen.push(format!("l{}", l.items.len())); false }
    pub fn opt_list(&mut self, l: &Option<crate::astdata::NodeList>) -> bool { self.seen.push(format!("ol{}", l.as_ref().map_or(0, |x| x.items.len()))); false }
    pub fn mods(&mut self, m: &Option<crate::astdata::ModifierList>) -> bool { self.seen.push(format!("m{}", m.as_ref().map_or(0, |x| x.flags.0))); false }
}

pub fn propagate_subtree_facts(child: Node) -> i32 { child.raw() as i32 }
pub fn propagate_node_list(children: DataList<'_>, f: fn(Node) -> i32) -> i32 {
    children.list.map_or(0, |l| l.items.iter().map(|i| f(Node::new(children.file, *i))).sum())
}
pub fn propagate_eraseable(child: Node) -> i32 { -(child.raw() as i32) }
pub fn propagate_eraseable_list(children: DataList<'_>) -> i32 { -(children.len() as i32) }

fn id(n: u32) -> NodeId { NodeId(n) }

fn walk_children(data: &NodeData, w: &mut Recorder) -> bool {
    macro_rules! n {
        ($x:expr) => {
            w.node($x)
        };
    }
    macro_rules! o {
        ($x:expr) => {
            w.opt($x)
        };
    }
    macro_rules! l {
        ($x:expr) => {
            w.list(&$x)
        };
    }
    macro_rules! ol {
        ($x:expr) => {
            w.opt_list(&$x)
        };
    }
    macro_rules! m {
        ($x:expr) => {
            w.mods(&$x)
        };
    }
    match data {
        NodeData::QualifiedName(d) => n!(d.left) || n!(d.right),
        NodeData::ComputedPropertyName(d) => n!(d.expression),
        NodeData::IfStatement(d) => n!(d.expression) || n!(d.then_statement) || o!(d.else_statement),
        NodeData::FunctionDeclaration(d) => o!(d.name) || l!(d.parameters) || m!(d.modifiers),
        NodeData::CallExpression(d) => n!(d.expression) || ol!(d.type_arguments),
        _ => false,
    }
}

fn facts_of_data(n: Node, d: &NodeData) -> i32 {
    let f = n.file_index();
    macro_rules! p {
        ($x:expr) => {
            propagate_subtree_facts(req(f, $x))
        };
    }
    macro_rules! po {
        ($x:expr) => {
            propagate_subtree_facts(opt(f, $x))
        };
    }
    macro_rules! pl {
        ($x:expr) => {
            propagate_node_list(DataList::req(f, &$x), propagate_subtree_facts)
        };
    }
    macro_rules! pol {
        ($x:expr) => {
            propagate_node_list(DataList::opt(f, &$x), propagate_subtree_facts)
        };
    }
    macro_rules! pm {
        ($x:expr) => {
            propagate_node_list(DataList::mods(f, &$x), propagate_subtree_facts)
        };
    }
    macro_rules! el {
        ($x:expr) => {
            propagate_eraseable_list(DataList::opt(f, &$x))
        };
    }
    macro_rules! e {
        ($x:expr) => {
            propagate_eraseable(opt(f, $x))
        };
    }
    match d {
        NodeData::BinaryExpression(d) => p!(d.left) + p!(d.right),
        NodeData::VariableDeclaration(d) => p!(d.name) + po!(d.initializer),
        NodeData::FunctionDeclaration(d) => pl!(d.parameters) + pm!(d.modifiers),
        NodeData::CallExpression(d) => p!(d.expression) + pol!(d.type_arguments),
        NodeData::TypeReferenceNode(d) => el!(d.type_arguments),
        NodeData::ParameterDeclaration(d) => e!(d.initializer),
        _ => 0,
    }
}

fn main() {
    let qn = NodeData::QualifiedName(Box::new(QualifiedNameData { left: id(1), right: id(2), ..Default::default() }));
    let mut w = Recorder { seen: Vec::new() };
    walk_children(&qn, &mut w);
    let fd = NodeData::FunctionDeclaration(Box::new(FunctionDeclarationData {
        name: Some(id(5)),
        parameters: crate::astdata::NodeList { items: vec![id(7), id(8)] },
        modifiers: Some(crate::astdata::ModifierList { list: crate::astdata::NodeList::default(), flags: crate::astdata::ModFlags(3) }),
        ..Default::default()
    }));
    walk_children(&fd, &mut w);
    let call = NodeData::CallExpression(Box::new(CallExpressionData {
        expression: id(9), type_arguments: Some(crate::astdata::NodeList { items: vec![id(1)] }), ..Default::default()
    }));
    walk_children(&call, &mut w);
    println!("walk={:?}", w.seen);

    let n0 = Node::new(0, id(0));
    let bin = NodeData::BinaryExpression(Box::new(BinaryExpressionData { left: id(3), right: id(4), ..Default::default() }));
    let vd = NodeData::VariableDeclaration(Box::new(VariableDeclarationData { name: id(5), initializer: Some(id(6)), ..Default::default() }));
    let tr = NodeData::TypeReferenceNode(Box::new(TypeReferenceNodeData {
        type_name: id(2), type_arguments: Some(crate::astdata::NodeList { items: vec![id(1), id(2), id(3)] }), ..Default::default()
    }));
    let pd = NodeData::ParameterDeclaration(Box::new(ParameterDeclarationData { name: id(1), initializer: Some(id(10)), ..Default::default() }));
    println!("facts bin={} vd={} fd={} call={} tr={} pd={}",
        facts_of_data(n0, &bin), facts_of_data(n0, &vd), facts_of_data(n0, &fd),
        facts_of_data(n0, &call), facts_of_data(n0, &tr), facts_of_data(n0, &pd));
}
