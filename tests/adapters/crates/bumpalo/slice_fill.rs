// bumpalo 3.20.3 slice fills: try_alloc_slice_fill_with as ts-rust's
// leak_arena.rs calls it (a `&mut F` closure, the Err fallback to a new
// chunk), allocation limits, a closure that allocates in the same Bump,
// and a panicking closure: the room comes back and the items made so
// far never drop.
use bumpalo::Bump;
use std::panic::{AssertUnwindSafe, catch_unwind};

struct Noisy(u32);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn fill<T>(b: &Bump, len: usize, mut make: impl FnMut(usize) -> T) -> &mut [T] {
    match b.try_alloc_slice_fill_with(len, &mut make) {
        Ok(s) => s,
        Err(_) => {
            println!("full at len {len}");
            let fresh: &'static Bump = Box::leak(Box::new(Bump::new()));
            fresh.alloc_slice_fill_with(len, make)
        }
    }
}

fn main() {
    let b = Bump::with_capacity(256);
    b.set_allocation_limit(Some(b.allocated_bytes()));
    println!("start {} {}", b.allocated_bytes(), b.chunk_capacity());
    for len in [0usize, 1, 7, 30, 64, 3, 300, 2] {
        let s = fill(&b, len, |i| (i * 3) as u16);
        println!("len {len}: {:?} left {}", &s[..s.len().min(4)], b.chunk_capacity());
    }
    let r = b.try_alloc_slice_fill_with(1000, |i| i as u64);
    println!("too big: {:?} left {}", r.map(|s| s.len()), b.chunk_capacity());
    let r = b.try_alloc_slice_fill_with(usize::MAX / 2, |i| i as u64);
    println!("layout: {:?}", r.map(|s| s.len()));

    // The closure allocates in the same Bump: the slice's room comes first.
    let n = Bump::new();
    let s = n.try_alloc_slice_fill_with(4, |i| *n.alloc(i as u32 + 10)).unwrap();
    println!("nested {:?} left {} total {}", s, n.chunk_capacity(), n.allocated_bytes());

    // A panic mid-fill, in the same chunk: the room comes back.
    let p = Bump::new();
    p.alloc(1u64);
    let before = p.chunk_capacity();
    let r = catch_unwind(AssertUnwindSafe(|| {
        p.try_alloc_slice_fill_with(5, |i| {
            if i == 3 {
                panic!("stop at {i}");
            }
            Noisy(i as u32)
        })
        .map(|s| s.len())
    }));
    println!("panicked {} before {before} after {}", r.is_err(), p.chunk_capacity());

    // A panic in a fill that made its own chunk: the chunk start.
    let q = Bump::with_capacity(64);
    q.alloc([0u8; 40]);
    let r = catch_unwind(AssertUnwindSafe(|| {
        q.alloc_slice_fill_with(100, |i| {
            if i == 50 {
                panic!("big stop");
            }
            i as u64
        })
        .len()
    }));
    println!("panicked {} left {} total {}", r.is_err(), q.chunk_capacity(), q.allocated_bytes());

    // A panic after a nested allocation: nothing comes back.
    let m = Bump::new();
    let r = catch_unwind(AssertUnwindSafe(|| {
        m.alloc_slice_fill_with(3, |i| {
            m.alloc(i as u8);
            if i == 1 {
                panic!("nested stop");
            }
            i
        })
        .len()
    }));
    println!("panicked {} left {}", r.is_err(), m.chunk_capacity());
}
