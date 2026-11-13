// ts-rust ast/synthetic.rs set_node_modifiers and parser/reparser.rs mutate:
// a std trait method by path (`Clone::clone`) passed for an `impl
// FnOnce(&T) -> R` bound is `for<'r> fn(&'r T) -> T`.
#[derive(Clone, Debug)]
struct NodeData {
    n: u32,
}
fn with_ast_data<R>(d: &NodeData, f: impl FnOnce(&NodeData) -> R) -> R {
    f(d)
}
fn main() {
    let d = NodeData { n: 3 };
    let mut c = with_ast_data(&d, Clone::clone);
    c.n += 1;
    println!("{:?} {:?}", d, c);
}
