// A by-value parameter over 16 bytes (release C: a pointer to a fresh copy, copied in on
// entry) with drop glue, in callees that mutate it, move part of it out, or replace a field,
// then panic or return: every D drops exactly once on both paths.
use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};

thread_local! {
    static DROPS: Cell<u32> = const { Cell::new(0) };
}

struct D(u32);

impl Drop for D {
    fn drop(&mut self) {
        DROPS.with(|d| d.set(d.get() + 1));
    }
}

struct Big {
    a: D,
    b: D,
    n: [u64; 3],
}

#[inline(never)]
fn mutate(mut b: Big, p: bool) -> u64 {
    b.n[0] += 1;
    if p {
        panic!("mutate");
    }
    b.n[0] + b.a.0 as u64
}

#[inline(never)]
fn partial(b: Big, p: bool) -> u32 {
    let a = b.a;
    if p {
        panic!("partial");
    }
    a.0 + b.b.0
}

#[inline(never)]
fn replace(mut b: Big, p: bool) -> u32 {
    b.a = D(100);
    if p {
        panic!("replace");
    }
    b.a.0 + b.b.0
}

fn drops() -> u32 {
    DROPS.with(|d| d.get())
}

fn mk(i: u32) -> Big {
    Big { a: D(i), b: D(i + 1), n: [i as u64; 3] }
}

fn main() {
    for p in [false, true] {
        let r = catch_unwind(AssertUnwindSafe(|| mutate(mk(1), p)));
        println!("mutate {p}: {:?}, drops {}", r.ok(), drops());
        let r = catch_unwind(AssertUnwindSafe(|| partial(mk(2), p)));
        println!("partial {p}: {:?}, drops {}", r.ok(), drops());
        let r = catch_unwind(AssertUnwindSafe(|| replace(mk(3), p)));
        println!("replace {p}: {:?}, drops {}", r.ok(), drops());
    }
    let f: fn(Big, bool) -> u32 = replace;
    let r = catch_unwind(AssertUnwindSafe(|| f(mk(4), true)));
    println!("through a fn pointer: {:?}, drops {}", r.ok(), drops());
}
