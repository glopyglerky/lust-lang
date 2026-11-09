// Calls through `dyn Fn*` values: every region of the signature is
// late-bound, so an argument may be a local of the caller, the result
// borrows only the one input, and a closure coerced to the dyn type may
// return a borrow of its argument.
use std::rc::Rc;

struct Tok<'a>(&'a str);

struct Emitter {
    out: Vec<String>,
    cb: Box<dyn FnMut(&str) -> usize>,
}

impl Emitter {
    fn emit(&mut self, s: &str) {
        let n = (self.cb)(s);
        self.out.push(format!("{}:{}", s, n));
    }
}

#[derive(Clone, Copy, Debug)]
struct Node(u32);

struct NodeVisitor<'a> {
    visit: Rc<dyn Fn(Node, &mut NodeVisitor) -> Node + 'a>,
    depth: u32,
}

fn walk(v: &mut NodeVisitor, n: Node) -> Node {
    v.depth += 1;
    let f = v.visit.clone();
    f(n, v)
}

fn call(f: &mut dyn FnMut(&u8)) {
    let l = 5u8;
    f(&l);
}

fn through<F: Fn(&u8) -> &u8>(g: &F) -> u8 {
    let d: &dyn Fn(&u8) -> &u8 = g;
    let l = 1u8;
    *d(&l)
}

fn main() {
    let mut n = 0u32;
    let mut g = |x: &u8| n += *x as u32;
    call(&mut g);
    println!("{}", n);
    let f: &dyn Fn(&u8) -> &u8 = &|x| x;
    let k = 1u8;
    println!("{}", f(&k));
    let t: Box<dyn Fn(&str) -> Tok> = Box::new(|s| Tok(s));
    let s = String::from("ab");
    println!("{}", t(&s).0);
    let h: Box<dyn Fn(&[u8]) -> Option<&u8>> = Box::new(|v| v.first());
    println!("{:?}", h(&[4u8, 5]));
    let mut total = 0usize;
    let mut e = Emitter { out: Vec::new(), cb: Box::new(move |s| {
        total += s.len();
        total
    }) };
    e.emit("ab");
    e.emit("cde");
    println!("{:?}", e.out);
    let offset = 10;
    let mut v = NodeVisitor {
        visit: Rc::new(|n: Node, vis: &mut NodeVisitor| if n.0 < 3 { walk(vis, Node(n.0 + 1)) } else { Node(n.0 + offset) }),
        depth: 0,
    };
    let out = walk(&mut v, Node(0));
    println!("{:?} {}", out, v.depth);
    println!("{}", through(&|x: &u8| -> &u8 { x }));
}
