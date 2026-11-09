// Arguments over 16 bytes go by pointer to a fresh copy in release C, and a callee whose unwind
// cleanup can drop its parameter uses that copy in place (cgen `FnGen::function`, `in_place`).
// Every D drops exactly once when an argument read panics before the call (the moved value drops
// in the caller), when the callee moves part of its parameter out and replaces another part, then
// panics, along a recursive chain that forwards its parameter and panics at the bottom, and when
// a moved local is assigned again in a loop.
use std::cell::Cell;
use std::hint::black_box;
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

fn drops() -> u32 {
    DROPS.with(|d| d.get())
}

fn mk(i: u32) -> Big {
    Big { a: D(i), b: D(i + 1), n: [i as u64; 3] }
}

#[inline(never)]
fn take(b: Big, k: u64) -> u64 {
    b.n[0] + k + b.a.0 as u64
}

#[inline(never)]
fn part_replace(mut b: Big, p: bool) -> u32 {
    let a = b.a;
    b.b = D(50);
    b.n[2] += 1;
    if p {
        panic!("part_replace");
    }
    a.0 + b.b.0 + b.n[2] as u32
}

#[inline(never)]
fn chain(b: Big, depth: u32, p: bool) -> u64 {
    if depth == 0 {
        if p {
            panic!("bottom");
        }
        return b.n[1] + b.b.0 as u64;
    }
    chain(b, depth - 1, p)
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let i = black_box(5usize);
    let r = catch_unwind(AssertUnwindSafe(|| {
        let b = mk(1);
        let a = [1u64];
        take(b, a[i])
    }));
    println!("argument panic: {:?}, drops {}", r.ok(), drops());
    let r = catch_unwind(AssertUnwindSafe(|| {
        let b = mk(1);
        let a = [1u64];
        take(b, a[black_box(0)])
    }));
    println!("argument ok: {:?}, drops {}", r.ok(), drops());
    for p in [false, true] {
        let r = catch_unwind(AssertUnwindSafe(|| part_replace(mk(10), p)));
        println!("part_replace {p}: {:?}, drops {}", r.ok(), drops());
        let r = catch_unwind(AssertUnwindSafe(|| chain(mk(20), 3, p)));
        println!("chain {p}: {:?}, drops {}", r.ok(), drops());
    }
    let mut b = mk(30);
    let mut sum = 0;
    for k in 0..4u64 {
        sum += take(b, k);
        b = mk(31 + k as u32);
    }
    println!("loop: {sum}, drops {}", drops());
    drop(b);
    println!("end: drops {}", drops());
}
