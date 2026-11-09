// A generic recursive type through `Box<[Self]>` with derived Clone: the
// Clone glue walk must meet `Tree<u32>` on its stack again (lust overflowed
// its own stack in build).
#[derive(Debug, Clone, PartialEq)]
enum Tree<T> {
    Leaf(T),
    Node(Box<[Tree<T>]>),
}

fn sum<T: Copy + Into<u64>>(t: &Tree<T>) -> u64 {
    match t {
        Tree::Leaf(v) => (*v).into(),
        Tree::Node(kids) => kids.iter().map(sum).sum(),
    }
}

fn main() {
    let leaf = Tree::Leaf(3u32);
    let t = Tree::Node(vec![leaf.clone(), Tree::Node(vec![Tree::Leaf(4)].into_boxed_slice())].into_boxed_slice());
    let c = t.clone();
    println!("{} {}", sum(&c), c == t);
    println!("{c:?}");
    let s: Tree<u8> = Tree::Node(Box::new([]));
    println!("{:?} {}", s.clone(), sum(&s));
}
