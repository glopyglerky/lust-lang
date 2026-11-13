// A const fn whose result type could hold a `&mut` (a field of type
// `Option<&mut u32>` behind a shared reference) but whose value holds no
// reference: the evaluator takes it (ts-rust ast/synthetic.rs ARENA).
use std::cell::RefCell;
struct Prog {
    w: Option<&'static mut u32>,
    id: u32,
}
struct Arena {
    v: Vec<u32>,
    n: usize,
    last: Option<(&'static Prog, u32)>,
}
impl Arena {
    const fn new() -> Self {
        Self { v: Vec::new(), n: 1, last: None }
    }
    fn push(&mut self, x: u32) {
        self.v.push(x);
        self.n += 1;
    }
}
thread_local! {
    static ARENA: RefCell<Arena> = const { RefCell::new(Arena::new()) };
}
struct W<'a> {
    r: Option<&'a mut u32>,
    k: u32,
}
const fn none() -> W<'static> {
    W { r: None, k: 7 }
}
const fn take(w: W<'_>, add: u32) -> u32 {
    match w.r {
        Some(r) => *r + add,
        None => w.k + add,
    }
}
const D: u32 = take(none(), 3);
fn main() {
    ARENA.with(|a| a.borrow_mut().push(3));
    ARENA.with(|a| {
        let a = a.borrow();
        println!("{} {} {}", a.n, a.v.len(), a.last.map(|(p, _)| (p.id, p.w.is_some())).is_none());
    });
    println!("{}", D);
}
