use std::cell::Cell;
use std::sync::atomic::{AtomicU32, Ordering};
static TOP_A: AtomicU32 = AtomicU32::new(10);
fn early() -> u32 {
    static E: AtomicU32 = AtomicU32::new(7);
    E.fetch_add(3, Ordering::Relaxed) + 3
}
fn inner_tls() -> u32 {
    thread_local! {
        static T: Cell<u32> = Cell::new(40);
    }
    T.with(|t| {
        t.set(t.get() + 2);
        t.get()
    })
}
fn bump() -> u32 {
    static HITS: AtomicU32 = AtomicU32::new(0);
    HITS.fetch_add(1, Ordering::Relaxed) + 1
}
fn label2() -> &'static str {
    static NAME: &str = "other";
    NAME
}
fn label() -> &'static str {
    static NAME: &str = "second";
    NAME
}
fn shape() -> u32 {
    struct P {
        x: u32,
        y: u32,
    }
    let p = P { x: 4, y: 5 };
    p.x * 10 + p.y
}
thread_local! {
    static DEPTH: Cell<u32> = const { Cell::new(5) };
}
fn depth() -> u32 {
    DEPTH.with(|d| {
        d.set(d.get() + 1);
        d.get()
    })
}
fn main() {
    println!("{} {} {} {}", TOP_A.load(Ordering::Relaxed), bump(), bump(), label());
    println!("{} {}", depth(), depth());
    println!("{} {}", early(), early());
    println!("{}", shape());
    println!("{} {}", label2(), label());
    println!("{} {}", inner_tls(), inner_tls());
}
