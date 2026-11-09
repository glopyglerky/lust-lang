// Recursive types reached through hash, B-tree, and deque containers, as
// keys and values, with entries and iterators naming the helper types
// first. Every C struct must come after the types it holds by value only.
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

// A HashSet of itself: Hash and Eq by hand (a HashSet has no Hash).
#[derive(Debug, Clone)]
enum K {
    Leaf(u32),
    Set(HashSet<K>),
}
impl PartialEq for K {
    fn eq(&self, o: &K) -> bool {
        self.depth() == o.depth()
    }
}
impl Eq for K {}
impl Hash for K {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.depth().hash(h)
    }
}
impl K {
    fn depth(&self) -> u32 {
        match self {
            K::Leaf(_) => 0,
            K::Set(s) => 1 + s.iter().map(K::depth).max().unwrap_or(0),
        }
    }
}

// Mutual recursion through a map value and a deque.
#[derive(Debug, Clone, PartialEq)]
struct Dir {
    files: BTreeMap<String, Node>,
}
#[derive(Debug, Clone, PartialEq)]
enum Node {
    File(u32),
    Dir(Dir),
    Queue(VecDeque<(u8, Node)>),
}

// A generic tree through a map of Rc slices.
#[derive(Debug)]
enum Tree<T> {
    Leaf(T),
    Kids(HashMap<u8, Rc<[Tree<T>]>>),
}

fn size(n: &Node) -> u32 {
    match n {
        Node::File(s) => *s,
        Node::Dir(d) => d.files.values().map(size).sum(),
        Node::Queue(q) => q.iter().map(|(_, n)| size(n)).sum(),
    }
}

fn sum<T: Copy + Into<u64>>(t: &Tree<T>) -> u64 {
    match t {
        Tree::Leaf(v) => (*v).into(),
        Tree::Kids(m) => m.values().flat_map(|s| s.iter()).map(sum).sum(),
    }
}

fn main() {
    let mut ks: HashMap<K, u32> = HashMap::new();
    *ks.entry(K::Leaf(1)).or_insert(0) += 1;
    *ks.entry(K::Set(HashSet::from([K::Leaf(2)]))).or_insert(0) += 5;
    *ks.entry(K::Leaf(9)).or_insert(0) += 1;
    let mut counts: Vec<(u32, u32)> = ks.iter().map(|(k, n)| (k.depth(), *n)).collect();
    counts.sort();
    println!("{counts:?}");

    let mut root = Dir { files: BTreeMap::new() };
    root.files.insert("a".to_string(), Node::File(10));
    let sub = Dir { files: BTreeMap::from([("b".to_string(), Node::File(5))]) };
    root.files.insert("sub".to_string(), Node::Dir(sub));
    let mut q = VecDeque::new();
    q.push_back((1, Node::File(2)));
    q.push_front((0, Node::Dir(root.clone())));
    root.files.entry("q".to_string()).or_insert(Node::Queue(q));
    if let Some(Node::Queue(q)) = root.files.get("q") {
        println!("queue front {:?}", q.front().map(|(t, n)| (*t, size(n))));
    }
    println!("total {}", size(&Node::Dir(root.clone())));
    println!("{}", root == root.clone());

    let leaves: Rc<[Tree<u32>]> = Rc::from(vec![Tree::Leaf(4), Tree::Leaf(6)]);
    let t = Tree::Kids(HashMap::from([(1, leaves)]));
    println!("tree {}", sum(&t));
    if let Tree::Kids(m) = &t {
        println!("{:?}", m.get(&1).map(|s| s.len()));
    }
}
