// Values built in their final place (release C, cgen's R1 and R3), with panics
// in a chain of large returns: a destination that held a value keeps it, nothing is dropped
// twice, and a value built in the caller's storage before the panic is not dropped by it.
use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};

thread_local! {
    static DROPS: Cell<u32> = const { Cell::new(0) };
}

struct Loud(u32, Vec<u32>, u64);

impl Drop for Loud {
    fn drop(&mut self) {
        DROPS.with(|d| d.set(d.get() + 1));
        println!("drop {} {:?}", self.0, self.1);
    }
}

#[derive(Clone, Copy, Debug)]
struct Big {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
    e: u64,
}

#[inline(never)]
fn check(n: u64) {
    if n == 3 {
        panic!("three");
    }
}

#[inline(never)]
fn half(n: u64) -> Big {
    let mut r = Big { a: n, b: 0, c: 0, d: 0, e: 0 };
    r.b = n * 2;
    check(n);
    r.c = n * 3;
    r
}

// Passes its out storage on to a callee that may panic.
#[inline(never)]
fn half_chain(n: u64) -> Big {
    half(n + 1)
}

#[inline(never)]
fn build(n: u32) -> Loud {
    let v = vec![n; 2];
    if n == 0 {
        panic!("zero");
    }
    Loud(n, v, n as u64)
}

#[inline(never)]
fn build_chain(n: u32) -> Loud {
    build(n)
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let mut b = half(1);
    let r = catch_unwind(AssertUnwindSafe(|| {
        b = half_chain(2);
    }));
    println!("{} {:?}", r.is_err(), b);
    let mut keep = build(1);
    let r = catch_unwind(AssertUnwindSafe(|| {
        keep = build_chain(0);
    }));
    println!("{} {}", r.is_err(), keep.0);
    let r = catch_unwind(|| build_chain(0).0);
    println!("{}", r.is_err());
    let r = catch_unwind(|| build_chain(4).0);
    println!("{r:?}");
    let r = catch_unwind(|| {
        let fresh = build_chain(6);
        let again = build_chain(0);
        fresh.0 + again.0
    });
    println!("{}", r.is_err());
    drop(keep);
    println!("drops {}", DROPS.with(|d| d.get()));
}
