// Arguments over 16 bytes go by pointer to a fresh copy in release C, and a callee whose unwind
// cleanup can drop its parameter uses that copy in place (cgen `FnGen::function`, `in_place`):
// moved named locals and moved temporaries (a struct literal, a call result), a callee that
// mutates its parameter, borrows it and passes the borrow on, moves part of it out, or moves it
// whole into a Vec; a `self` taken by value; a Copy argument passed while a shared borrow of it is
// live (a fresh copy, so the borrow sees the old value); recursion; fn pointers, dyn and threads.
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
struct Big {
    name: String,
    v: Vec<u32>,
    n: [u64; 2],
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Pod {
    a: [u64; 4],
}

fn mk(s: &str, k: u32) -> Big {
    Big { name: s.to_string(), v: (0..k).collect(), n: [k as u64, 2 * k as u64] }
}

#[inline(never)]
fn mutate(mut b: Big) -> String {
    b.v.push(99);
    b.n[0] += 1;
    b.name.push('!');
    format!("{} {:?} {:?}", b.name, b.v, b.n)
}

fn peek(b: &Big) -> usize {
    b.v.len() + b.name.len()
}

#[inline(never)]
fn borrow_on(b: Big) -> usize {
    let r = &b;
    peek(r) + peek(&b)
}

#[inline(never)]
fn part(b: Big) -> (String, usize) {
    let name = b.name;
    (name, b.v.len())
}

#[inline(never)]
fn into_vec(b: Big, out: &mut Vec<Big>) {
    out.push(b);
}

impl Big {
    #[inline(never)]
    fn consume(self) -> u64 {
        self.n[0] + self.n[1] + self.v.len() as u64
    }
}

#[inline(never)]
fn pod_sum(mut p: Pod) -> u64 {
    p.a[0] += 100;
    p.a.iter().sum()
}

fn rec(b: Big, d: u32) -> u64 {
    if d == 0 {
        return b.n[1];
    }
    rec(b, d - 1) + 1
}

trait Eat: Debug {
    fn eat(&self, b: Big) -> usize;
}

#[derive(Debug)]
struct E;

impl Eat for E {
    fn eat(&self, b: Big) -> usize {
        b.v.len() * 10 + b.name.len()
    }
}

fn main() {
    let b = mk("one", 3);
    println!("{}", mutate(b));
    println!("{}", mutate(Big { name: "lit".into(), v: vec![1], n: [5, 6] }));
    println!("{}", mutate(mk("call", 2)));
    let b = mk("two", 4);
    println!("{}", borrow_on(b));
    println!("{:?}", part(mk("three", 5)));
    let mut out = Vec::new();
    into_vec(mk("four", 1), &mut out);
    into_vec(mk("five", 2), &mut out);
    println!("{:?}", out);
    println!("{}", mk("six", 6).consume());
    let p = Pod { a: [1, 2, 3, 4] };
    let r = &p;
    println!("{} {:?} {:?}", pod_sum(p), r, p);
    println!("{}", rec(mk("rec", 7), 4));
    let f: fn(Big) -> String = mutate;
    println!("{}", f(mk("ptr", 1)));
    let e: &dyn Eat = &E;
    println!("{}", e.eat(mk("dyn", 3)));
    let t = std::thread::spawn(move || mutate(mk("thread", 2)));
    println!("{}", t.join().unwrap());
    let mut b = mk("loop", 0);
    for k in 1..4 {
        println!("{}", mutate(b));
        b = mk("loop", k);
    }
    println!("{:?}", b);
}
