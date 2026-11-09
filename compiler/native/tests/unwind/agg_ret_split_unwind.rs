// A Drop type returned through the out pointer (12 bytes, the pair at bytes 4-11), with panics
// before the result is written: no drop of the unwritten result, one per returned value.
use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};

thread_local! {
    static DROPS: Cell<u32> = const { Cell::new(0) };
}

#[derive(Debug)]
struct Held {
    a: u32,
    b: (u32, u32),
}

impl Drop for Held {
    fn drop(&mut self) {
        DROPS.with(|d| d.set(d.get() + 1));
    }
}

#[inline(never)]
fn held(x: u32) -> Held {
    if x == 5 {
        panic!("no Held for 5");
    }
    Held { a: x, b: (x + 1, x + 2) }
}

#[inline(never)]
fn pair(x: u32) -> Option<(u32, u32)> {
    if x == 2 {
        panic!("no pair for 2");
    }
    Some((x, x * 3))
}

fn through(f: fn(u32) -> Held, x: u32) -> Held {
    let h = f(x);
    Held { a: h.a + 1, b: h.b }
}

fn main() {
    let mut sum = 0;
    for x in 0..8 {
        match catch_unwind(AssertUnwindSafe(|| held(x))) {
            Ok(h) => sum += h.a + h.b.0 + h.b.1,
            Err(_) => println!("held {x} panicked"),
        }
    }
    println!("held sum {sum}, drops {}", DROPS.with(|d| d.get()));
    let r = catch_unwind(|| through(held, 5).a);
    println!("through {:?}, drops {}", r.is_err(), DROPS.with(|d| d.get()));
    println!("through {}, drops {}", through(held, 6).a, DROPS.with(|d| d.get()));
    for x in 0..4 {
        println!("pair {x}: {:?}", catch_unwind(|| pair(x)).ok());
    }
    let r = catch_unwind(|| {
        let v: Vec<Option<(u32, u32)>> = (3..6).map(pair).collect();
        if v.len() == 3 {
            panic!("after {v:?}");
        }
        v
    });
    println!("caught {}", r.is_err());
}
