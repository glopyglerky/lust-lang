// ARCH 2.6 and 9.3: the six ts-rust IntoIterator impl shapes,
// rebuilt with local types. Each concrete impl must coexist with
// the std blanket `impl<I: Iterator> IntoIterator for I`.
// ts-rust sources (read only):
//   ast/node.rs:1585 NodeSlice, ast/node.rs:1594 &NodeSlice,
//   checker/types.rs:289 &SharedList<T>, checker/types.rs:298 SharedList<T>,
//   core.rs:1134 Declarations, core.rs:1145 &Declarations.
// Node stands in for the ts-rust Node handle.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Node(u64);

// A plain iterator over a fixed array, shared by the shapes below.
#[derive(Clone)]
struct ArrIter {
    items: [Node; 3],
    next: usize,
}
impl ArrIter {
    fn new(items: [Node; 3]) -> Self {
        ArrIter { items, next: 0 }
    }
}
impl Iterator for ArrIter {
    type Item = Node;
    fn next(&mut self) -> Option<Node> {
        let item = self.items.get(self.next).copied();
        self.next += 1;
        item
    }
}
impl ExactSizeIterator for ArrIter {}

// Shapes 1 and 2: owned and borrowed impls for a slice-like type.
struct NodeSlice {
    items: [Node; 3],
}
impl NodeSlice {
    fn iter(&self) -> ArrIter {
        ArrIter::new(self.items)
    }
}
impl IntoIterator for NodeSlice {
    type Item = Node;
    type IntoIter = ArrIter;
    fn into_iter(self) -> ArrIter {
        self.iter()
    }
}
impl IntoIterator for &NodeSlice {
    type Item = Node;
    type IntoIter = ArrIter;
    fn into_iter(self) -> ArrIter {
        self.iter()
    }
}

// Shapes 3 and 4: borrowed impl yields &T via slice::Iter; the
// owned impl requires T: Copy and yields copies.
use std::rc::Rc;
struct SharedList<T: 'static> {
    items: Rc<[T]>,
}
impl<T> SharedList<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.items.iter()
    }
    fn len(&self) -> usize {
        self.items.len()
    }
}
impl<'a, T> IntoIterator for &'a SharedList<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
struct SharedListIter<T: 'static> {
    list: SharedList<T>,
    front: usize,
    back: usize,
}
impl<T: Copy> Iterator for SharedListIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.front >= self.back {
            return None;
        }
        let item = self.list.items[self.front];
        self.front += 1;
        Some(item)
    }
}
impl<T: Copy> IntoIterator for SharedList<T> {
    type Item = T;
    type IntoIter = SharedListIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        let back = self.len();
        SharedListIter { list: self, front: 0, back }
    }
}
impl<T> Clone for SharedList<T> {
    fn clone(&self) -> Self {
        SharedList { items: Rc::clone(&self.items) }
    }
}

// Shapes 5 and 6: an owned impl with a custom IntoIter struct and
// a borrowed impl that yields &Node via slice::Iter.
struct Declarations {
    items: [Node; 3],
}
impl Declarations {
    fn iter(&self) -> std::slice::Iter<'_, Node> {
        self.items.iter()
    }
}
struct DeclarationsIntoIter {
    list: Declarations,
    next: usize,
}
impl Iterator for DeclarationsIntoIter {
    type Item = Node;
    fn next(&mut self) -> Option<Node> {
        let item = self.list.items.get(self.next).copied();
        self.next += 1;
        item
    }
}
impl ExactSizeIterator for DeclarationsIntoIter {}
impl IntoIterator for Declarations {
    type Item = Node;
    type IntoIter = DeclarationsIntoIter;
    fn into_iter(self) -> DeclarationsIntoIter {
        DeclarationsIntoIter { list: self, next: 0 }
    }
}
impl<'a> IntoIterator for &'a Declarations {
    type Item = &'a Node;
    type IntoIter = std::slice::Iter<'a, Node>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

fn sum_nodes<I: IntoIterator<Item = Node>>(iter: I) -> u64 {
    iter.into_iter().map(|n| n.0).sum()
}
fn main() {
    let items = [Node(1), Node(2), Node(3)];
    let borrowed_slice = NodeSlice { items };
    let a = sum_nodes(&borrowed_slice);
    let b = sum_nodes(borrowed_slice);
    let shared: SharedList<u8> = SharedList { items: Rc::from([4u8, 5, 6]) };
    let c: u8 = (&shared).into_iter().sum();
    let d: u8 = shared.into_iter().sum();
    let decls = Declarations { items };
    let e: u64 = (&decls).into_iter().map(|n| n.0).sum();
    let f = sum_nodes(decls);
    println!("{a} {b} {c} {d} {e} {f}");
}
