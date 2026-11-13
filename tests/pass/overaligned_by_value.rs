// Values with alignment over 16 cross every C call boundary by value (fx-overalign):
// tcc misplaces such stack arguments, so lust passes them by pointer to a copy.
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Default, Hash, Eq, PartialOrd, Ord)]
#[repr(C, align(64))]
struct A {
    v: Vec<u32>,
    s: String,
    b: Box<u8>,
    n: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(align(32))]
struct C(u8, u16);

struct D(A);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop D {:?}", self.0.n);
    }
}

trait Tr {
    fn tk1(&self, a: A, c: C) -> A;
    fn tk2(&self, a: A) -> u32;
}
struct S(u32);
impl Tr for S {
    fn tk1(&self, mut a: A, c: C) -> A {
        a.n += self.0 + c.0 as u32 + c.1 as u32;
        a
    }
    fn tk2(&self, a: A) -> u32 {
        a.n + self.0
    }
}

fn mk(n: u32) -> A {
    A { v: vec![n, n + 1], s: format!("s{n}"), b: Box::new(n as u8), n }
}
fn pass(a: A) -> A {
    a
}
fn pass_g<T>(t: T) -> T {
    t
}
fn many(x1: u64, x2: u64, x3: u64, x4: u64, x5: u64, x6: u64, a: A, x7: u64, c: C, b: A) -> u64 {
    x1 + x2 + x3 + x4 + x5 + x6 + x7 + a.n as u64 + c.0 as u64 + b.v.len() as u64 + b.s.len() as u64
}
fn copy_mut(mut c: C) -> u8 {
    c.0 += 1;
    c.0
}
fn opt(o: Option<A>) -> Option<A> {
    o.map(|mut a| {
        a.n *= 2;
        a
    })
}
fn res(r: Result<A, C>) -> u32 {
    match r {
        Ok(a) => a.n,
        Err(c) => c.1 as u32,
    }
}
fn tup(t: (u8, A, C)) -> (C, A) {
    (t.2, t.1)
}
fn arr(a: [A; 2]) -> u32 {
    a[0].n + a[1].n
}
fn dyn_fn(f: &dyn Fn(A) -> u32, a: A) -> u32 {
    f(a)
}
fn dyn_fnmut(f: &mut dyn FnMut(A, C) -> u32) -> u32 {
    f(mk(4), C(1, 2))
}
fn boxed_once(f: Box<dyn FnOnce(A) -> A>) -> A {
    f(mk(5))
}
fn impl_fn(f: impl Fn(A) -> A) -> A {
    f(mk(6))
}
fn ptr(f: fn(A) -> A) -> A {
    f(mk(7))
}

fn main() {
    let a = pass(mk(1));
    println!("{:?}", a);
    let g = pass_g(mk(2));
    println!("{} {:?}", g.n, pass_g(C(3, 4)));
    let c = C(9, 10);
    println!("{} {:?}", copy_mut(c), c);
    println!("{}", many(1, 2, 3, 4, 5, 6, mk(8), 7, C(1, 1), mk(9)));
    println!("{:?}", opt(Some(mk(10))).map(|a| a.n));
    println!("{} {}", res(Ok(mk(11))), res(Err(C(0, 12))));
    let (cc, aa) = tup((1, mk(13), C(2, 3)));
    println!("{:?} {}", cc, aa.n);
    println!("{}", arr([mk(14), mk(15)]));
    let k = 3;
    println!("{}", dyn_fn(&|a: A| a.n + k, mk(16)));
    let mut tot = 0;
    println!("{}", dyn_fnmut(&mut |a: A, c: C| {
        tot += a.n + c.1 as u32;
        tot
    }));
    let cap = mk(17);
    println!("{:?}", boxed_once(Box::new(move |a: A| A { n: a.n + cap.n, ..a })).n);
    let cap2 = mk(18);
    println!("{}", impl_fn(move |a| A { n: a.n + cap2.n, ..a }).n);
    println!("{}", ptr(pass).n);
    let s: Box<dyn Tr> = Box::new(S(100));
    println!("{}", s.tk1(mk(19), C(1, 1)).n);
    println!("{}", s.tk2(mk(20)));
    // Box/Rc/Arc/Vec moves
    let bx = Box::new(mk(21));
    let rc = Rc::new(mk(22));
    let ar = Arc::new(mk(23));
    let mut vs = vec![mk(24)];
    vs.push(mk(25));
    vs.insert(0, mk(26));
    let popped = vs.pop().unwrap();
    println!("{} {} {} {} {}", bx.n, rc.n, ar.n, popped.n, vs.iter().map(|a| a.n).sum::<u32>());
    let ub = *bx;
    println!("{}", ub.n);
    // closure captures, move closures returning captured value
    let cap3 = mk(27);
    let f = move || cap3;
    println!("{}", f().n);
    let mk_with = |g: &dyn Fn() -> A| g();
    println!("{}", mk_with(&|| mk(28)).n);
    // drop and RefCell / Mutex / OnceLock
    {
        let _d = D(mk(29));
        let d2 = D(mk(30));
        let moved = pass_g(d2);
        println!("moved {}", moved.0.n);
    }
    let cell = RefCell::new(mk(31));
    let old = cell.replace(mk(32));
    println!("{} {}", old.n, cell.borrow().n);
    let m = Mutex::new(mk(33));
    *m.lock().unwrap() = mk(34);
    println!("{}", m.into_inner().unwrap().n);
    static ONCE: OnceLock<C> = OnceLock::new();
    println!("{:?}", ONCE.get_or_init(|| C(5, 6)));
    let mut hm: HashMap<u32, A> = HashMap::new();
    hm.insert(1, mk(35));
    hm.insert(2, mk(36));
    println!("{:?}", hm.remove(&1).map(|a| a.n));
    // threads
    let t = std::thread::spawn(move || {
        let a = mk(37);
        pass(a)
    });
    println!("{}", t.join().unwrap().n);
    let shared = mk(38);
    let r = std::thread::scope(|sc| sc.spawn(|| shared.n + 1).join().unwrap());
    println!("{}", r);
    // iterator adapters
    let total: u32 = (0..3).map(mk).map(pass).filter(|a| a.n > 0).map(|a| a.n).sum();
    println!("{}", total);
    let mut sorted = vec![mk(3), mk(1), mk(2)];
    sorted.sort();
    sorted.sort_by_key(|a| std::cmp::Reverse(a.n));
    println!("{:?}", sorted.iter().map(|a| a.n).collect::<Vec<_>>());
    let folded = (0..4).fold(mk(0), |mut acc, i| {
        acc.n += i;
        acc
    });
    println!("{}", folded.n);
    let d = A::default();
    println!("{:?} {} {}", d.clone() == d, std::mem::align_of::<A>(), std::mem::size_of::<A>());
}
