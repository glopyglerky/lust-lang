// `&&Vec<T>` coerces to `&[T]` through both derefs (ts-rust core.rs
// `impl Deref for Declarations`: a match arm binds `list: &&'static
// Vec<Node>` and returns it as `&[Node]`; here the list is borrowed
// for `'a`), also in returns that name the inner or outer lifetime, and
// in closures.
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
struct Node(u32);

enum List<'a> {
    Empty,
    One(Node),
    Many(Arc<Vec<Node>>),
    Borrowed(&'a Vec<Node>),
}

struct Decls<'a>(List<'a>);

impl std::ops::Deref for Decls<'_> {
    type Target = [Node];
    fn deref(&self) -> &[Node] {
        match &self.0 {
            List::Empty => &[],
            List::One(node) => std::slice::from_ref(node),
            List::Many(list) => list,
            List::Borrowed(list) => list,
        }
    }
}

fn inner<'a>(x: &&'a Vec<u8>) -> &'a [u8] {
    x
}

fn outer<'a, 'b: 'a>(x: &'a &'b mut Vec<u8>) -> &'a [u8] {
    x
}

fn main() {
    let v = vec![Node(1), Node(2)];
    let d = Decls(List::Borrowed(&v));
    let e = Decls(List::Many(Arc::new(vec![Node(3)])));
    let f = Decls(List::One(Node(4)));
    let g = Decls(List::Empty);
    println!("{:?} {} {:?} {:?} {:?}", &*d, d.len(), &*e, &*f, &*g);
    let w = vec![5u8, 6];
    let r = &w;
    let s: &[u8] = &r;
    let t = { let rr = &r; inner(rr) };
    let mut m = vec![7u8];
    let rm = &mut m;
    println!("{:?} {:?} {:?}", s, t, outer(&rm));
    let count = || {
        let s: &[u8] = &r;
        s.len()
    };
    println!("{}", count());
}
