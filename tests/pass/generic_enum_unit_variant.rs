// A unit variant of a generic enum takes its params from a turbofish
// or from later uses.
#[derive(Debug)]
enum Tree<T> {
    Leaf,
    Node(Box<Tree<T>>, T),
}

fn depth<T>(t: &Tree<T>) -> u32 {
    match t {
        Tree::Leaf => 0,
        Tree::Node(inner, _) => 1 + depth(inner),
    }
}

fn main() {
    let a: Tree<u8> = Tree::Leaf;
    let b = Tree::Node(Box::new(Tree::Leaf), 'x');
    let c = Tree::<i64>::Leaf;
    let d = Tree::Node(Box::new(Tree::Node(Box::new(Tree::Leaf), 1u16)), 2);
    println!("{:?} {:?} {:?} {}", a, b, c, depth(&d));
}
