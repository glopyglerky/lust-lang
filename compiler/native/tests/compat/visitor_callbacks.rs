// K1 compatibility fixture: NodeVisitor with stored Rc<dyn Fn>
// callbacks and a borrowed visitor (lifetime 'a, not 'static).
// panic-mode: abort
//
// Real shape: crates/ts_goport/src/ast/visitor.rs NodeVisitor<'a, C>
// with VisitFn<'a, C> = Rc<dyn Fn(Node, &mut NodeVisitor<'a, C>) ->
// Node + 'a>, a hooks struct, a borrowed factory (&'a NodeFactory)
// and owned ctx state. This is also the K8 hard case: a callback
// that captures a short-lived borrow (the prefix below) and calls
// back into the visitor through the Rc it is stored in.
//
// Reductions, shape only: Node is an index into a local arena vec,
// the factory is a counter, hooks keep two of the nine fields.

use std::rc::Rc;

/// A node handle (ts-rust Node is an arena handle too).
type Node = u32;

#[derive(Clone, Debug)]
struct NodeData {
    name: String,
    children: Vec<Node>,
}

/// Makes replacement nodes during a visit (ts-rust NodeFactory).
#[derive(Default)]
struct NodeFactory {
    made: std::cell::Cell<u32>,
}

impl NodeFactory {
    fn rename(&self, data: &NodeData, prefix: &str) -> NodeData {
        self.made.set(self.made.get() + 1);
        NodeData {
            name: format!("{prefix}{}", data.name),
            children: data.children.clone(),
        }
    }
}

/// Go NodeVisitor.Visit (ts-rust visitor.rs, verbatim type shape).
type VisitFn<'a, C> = Rc<dyn Fn(Node, &mut NodeVisitor<'a, C>) -> Node + 'a>;
/// A hook over one node.
type VisitNodeHook<'a, C> = Rc<dyn Fn(Node, &mut NodeVisitor<'a, C>) -> Node + 'a>;

/// Hooks that override visiting (two of the nine ts-rust fields).
struct NodeVisitorHooks<'a, C> {
    visit_node: Option<VisitNodeHook<'a, C>>,
    visit_token: Option<VisitNodeHook<'a, C>>,
}

impl<C> Default for NodeVisitorHooks<'_, C> {
    fn default() -> Self {
        Self {
            visit_node: None,
            visit_token: None,
        }
    }
}

/// Visit state the callbacks mutate through the visitor (ctx in
/// ts-rust is often a &mut Checker; here a log and the arena).
struct Ctx<'x> {
    arena: &'x mut Vec<NodeData>,
    log: &'x mut Vec<String>,
}

/// Go ast.NodeVisitor (ts-rust visitor.rs field shapes).
struct NodeVisitor<'a, C = ()> {
    visit: Option<VisitFn<'a, C>>,
    factory: Option<&'a NodeFactory>,
    default_factory: NodeFactory,
    hooks: NodeVisitorHooks<'a, C>,
    ctx: C,
}

impl<'a, C> NodeVisitor<'a, C> {
    fn factory(&self) -> &NodeFactory {
        self.factory.unwrap_or(&self.default_factory)
    }

    /// Call the stored visit callback. The Rc is cloned first so the
    /// callback can call back into this visitor (the ts-rust pattern).
    fn visit_node(&mut self, node: Node) -> Node {
        if let Some(hook) = self.hooks.visit_node.clone() {
            return hook(node, self);
        }
        match self.visit.clone() {
            Some(f) => f(node, self),
            None => node,
        }
    }
}

impl<'a, 'x> NodeVisitor<'a, Ctx<'x>> {
    /// Visit each child and rebuild the child list (VisitEachChild).
    fn visit_each_child(&mut self, node: Node) -> Node {
        let children = self.ctx.arena[node as usize].children.clone();
        let mut out = Vec::new();
        for child in children {
            out.push(self.visit_node(child));
        }
        self.ctx.arena[node as usize].children = out;
        node
    }
}

fn print_tree(arena: &[NodeData], node: Node, depth: usize) {
    let data = &arena[node as usize];
    println!("{}{}", "  ".repeat(depth), data.name);
    for &child in &data.children {
        print_tree(arena, child, depth + 1);
    }
}

fn main() {
    // Tree: root(a(b, c), d(e))
    let mut arena: Vec<NodeData> = Vec::new();
    let mut push = |name: &str, children: Vec<Node>| {
        arena.push(NodeData {
            name: name.to_string(),
            children,
        });
        (arena.len() - 1) as Node
    };
    let b = push("b", vec![]);
    let c = push("c", vec![]);
    let a = push("a", vec![b, c]);
    let e = push("e", vec![]);
    let d = push("d", vec![e]);
    let root = push("root", vec![a, d]);

    let mut log: Vec<String> = Vec::new();
    let factory = NodeFactory::default();
    // A short-lived borrow captured by the stored callback: 'a is the
    // stack frame of main, not 'static (the K8 requirement).
    let prefix = String::from("v_");

    let mut visitor: NodeVisitor<'_, Ctx<'_>> = NodeVisitor {
        // Rename each node through the borrowed factory, recurse
        // through the visitor, record the order in the borrowed log.
        visit: Some(Rc::new(|node, v: &mut NodeVisitor<'_, Ctx<'_>>| {
            let renamed = {
                let data = &v.ctx.arena[node as usize];
                v.factory().rename(data, &prefix)
            };
            v.ctx.log.push(format!("visit {}", renamed.name));
            v.ctx.arena[node as usize] = renamed;
            v.visit_each_child(node)
        })),
        factory: Some(&factory),
        default_factory: NodeFactory::default(),
        hooks: NodeVisitorHooks {
            // A hook that intercepts one node kind by position: it
            // runs instead of visit for the root, then falls through
            // to the stored callback for children via visit_each_child.
            visit_node: Some(Rc::new(|node, v: &mut NodeVisitor<'_, Ctx<'_>>| {
                if node == root {
                    v.ctx.log.push("hook root".to_string());
                    v.hooks.visit_node = None;
                    let out = v.visit_node(node);
                    return out;
                }
                v.visit_each_child(node)
            })),
            visit_token: None,
        },
        ctx: Ctx {
            arena: &mut arena,
            log: &mut log,
        },
    };

    let out = visitor.visit_node(root);
    // ctx borrows end here; the visitor (and its Rc callbacks) die
    // with this frame.
    drop(visitor);

    println!("returned root: {out}");
    for line in &log {
        println!("{line}");
    }
    println!("factory made: {}", factory.made.get());
    print_tree(&arena, root, 0);
}
