// By-value parameters over 16 bytes (release C passes them by pointer to a fresh copy, which
// the callee copies in), next to a 16-byte one and a 9-byte Option the unwrap fast path reads:
// direct calls kept out of line, x = f(x), a Copy argument with a live borrow, fn pointers, dyn
// methods, closures (Fn, FnMut, FnOnce, a boxed FnOnce called through its vtable), generic fns,
// recursion, a tuple-struct constructor and Cell::set as fns, moved fields with a sibling left
// to drop, a moved Box pointee, and threads (a closure, a fn item, a boxed FnOnce).
use std::cell::Cell;
use std::hint::black_box;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq)]
struct P16 {
    a: u64,
    b: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct P17 {
    a: u64,
    b: u64,
    c: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Wide(u32, u32, u32, u32, u32);

#[derive(Debug, Clone, PartialEq)]
struct Big {
    name: String,
    xs: Vec<u32>,
    n: u64,
}

#[inline(never)]
fn sum16(p: P16) -> u64 {
    p.a + p.b
}

#[inline(never)]
fn sum17(p: P17) -> u64 {
    p.a + p.b + p.c as u64
}

#[inline(never)]
fn bump(mut p: P17) -> P17 {
    p.a += 1;
    p.c = p.c.wrapping_add(1);
    p
}

#[inline(never)]
fn take(b: Big) -> usize {
    b.name.len() + b.xs.len() + b.n as usize
}

#[inline(never)]
fn wide(w: Wide) -> u32 {
    w.0 + w.4
}

fn rec(p: P17, n: u32) -> u64 {
    if n == 0 { p.a } else { rec(P17 { a: p.a + 1, ..p }, n - 1) }
}

fn generic<T: Copy>(x: T, f: fn(T) -> u64) -> u64 {
    f(x)
}

trait Shape {
    fn area(&self, p: P17) -> u64;
}

struct Sq(u64);

impl Shape for Sq {
    fn area(&self, p: P17) -> u64 {
        self.0 * (p.a + p.b)
    }
}

struct Tup(P17, u8);

fn worker() -> u64 {
    sum17(P17 { a: 1, b: 1, c: 1 })
}

fn main() {
    let p16 = P16 { a: 1, b: 2 };
    let p17 = P17 { a: 3, b: 4, c: 5 };
    println!("{} {} {}", sum16(p16), sum17(p17), wide(Wide(1, 2, 3, 4, 5)));
    let opt = black_box(Some([7u8; 8]));
    println!("{:?} {:?}", opt.unwrap(), black_box(Some([1u8; 8])).expect("bytes"));
    let mut q = p17;
    for _ in 0..3 {
        q = bump(q);
    }
    println!("{q:?}");
    let r = &q;
    let s = sum17(q);
    println!("{} {}", s, r.a);
    let f: fn(P17) -> u64 = sum17;
    let fs: Vec<fn(P17) -> u64> = vec![sum17, |p| p.c as u64, f];
    println!("{:?}", fs.iter().map(|f| f(p17)).collect::<Vec<_>>());
    let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Sq(2)), Box::new(Sq(3))];
    println!("{:?}", shapes.iter().map(|s| s.area(p17)).collect::<Vec<_>>());
    let k = 10u64;
    let add = |p: P17| p.a + k;
    let mut total = 0;
    let mut acc = |p: P17| total += p.b;
    acc(p17);
    acc(p17);
    let env = Big { name: "env".into(), xs: vec![1], n: 2 };
    let consume = move |p: P17| p.a + k + env.n;
    println!("{} {} {}", add(p17), total, consume(p17));
    let dynf: &dyn Fn(P17) -> u64 = &add;
    let once: Box<dyn FnOnce(P17) -> u64> = Box::new(move |p| p.b + q.a);
    println!("{} {}", dynf(p17), once(p17));
    println!("{} {}", generic(p17, sum17), rec(p17, 5));
    let mk: fn(P17, u8) -> Tup = Tup;
    let t = mk(p17, 9);
    println!("{} {}", t.0.a, t.1);
    let c17 = Cell::new(p17);
    let set: fn(&Cell<P17>, P17) = Cell::set;
    set(&c17, P17 { a: 7, b: 8, c: 9 });
    println!("{:?}", c17.get());
    let big = Big { name: "abc".into(), xs: vec![1, 2, 3], n: 4 };
    let pair = (big.clone(), Big { name: "x".into(), xs: vec![], n: 0 });
    println!("{}", take(pair.0));
    println!("{}", pair.1.name);
    let boxed = Box::new(big);
    println!("{}", take(*boxed));
    let h = thread::spawn(move || sum17(p17) + take(Big { name: "t".into(), xs: vec![9], n: 1 }) as u64);
    println!("{}", h.join().unwrap());
    println!("{}", thread::spawn(worker).join().unwrap());
    let job: Box<dyn FnOnce() -> u64 + Send> = Box::new(move || sum17(p17) * 2);
    println!("{}", thread::spawn(job).join().unwrap());
}
