// Deep chains of builtin adapters whose `next`, `nth`, `count`, and
// `advance_by` call more than one op of the inner iterator. Lowering
// expands each op inline, so an arm that expanded an inner op twice made
// the code exponential in the chain depth: `skip(0)` x10 then `count()`
// took over 60 s to build (fx-traitdepth, fx-cgen-ice). Each arm now
// expands each inner op once, so this case builds fast. One fn per chain
// keeps each C function small for the sanitizer build.

fn skips(v: &[u32]) {
    let it = v.iter().skip(0).skip(1).skip(0).skip(2).skip(0).skip(1).skip(0).skip(0).skip(1).skip(0).skip(0).skip(1);
    println!("{}", it.clone().count());
    let mut s = it;
    println!("{:?}", s.nth(1));
}

fn peeks(v: &[u32]) {
    let it = v.iter().peekable().peekable().peekable().peekable().peekable().peekable().peekable().peekable().peekable().peekable();
    let mut p = it.peekable().peekable().peekable().peekable().peekable().peekable().peekable().peekable().peekable().peekable();
    println!("{:?}", p.peek());
    println!("{:?}", p.nth(2));
    println!("{}", p.count());
}

fn steps(v: &[u32]) {
    let it = v.iter().step_by(1).step_by(2).step_by(1).step_by(1).step_by(2).step_by(1).step_by(1).step_by(1).step_by(1).step_by(1).step_by(1).step_by(1);
    let mut st = it.clone();
    println!("{:?} {:?}", st.next(), st.nth(1));
    println!("{}", it.count());
}

fn takes(v: &[u32]) {
    let it = v.iter().take(19).take(18).take(17).take(16).take(15).take(14).take(13).take(12).take(11).take(10).take(9).take(8).take(7).take(6);
    let mut t = it;
    println!("{:?} {:?} {:?}", t.nth(3), t.nth(20), t.next());
}

fn filtered_skips(v: &[u32]) {
    let mut f = v.iter().skip(1).filter(|x| **x != 4).skip(0).filter(|_| true).skip(1).filter(|_| true).skip(0).filter(|_| true);
    println!("{:?}", f.nth(2));
}

fn main() {
    let v: Vec<u32> = (1..=20).collect();
    skips(&v);
    peeks(&v);
    steps(&v);
    takes(&v);
    filtered_skips(&v);
    // The overflow forms over a range (an O(1) inner `nth`).
    let mut m = (0..50u32).skip(usize::MAX - 1).peekable().take(4).skip(0).step_by(1).skip(1).peekable().skip(0);
    println!("{:?} {}", m.nth(3), m.count());
    let mut r = (0..1000u64).skip(3).step_by(2).skip(1).peekable().take(400).skip(2).step_by(3).skip(0).peekable();
    println!("{:?} {:?} {:?}", r.nth(usize::MAX), r.next(), r.nth(10));
    let mixed = (0..100u32).inspect(|x| if *x < 3 { println!("pull {x}") }).skip(1).peekable().step_by(2).skip(0).take(30).skip(2);
    println!("{}", mixed.count());
}
