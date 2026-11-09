// bumpalo 3.20.3 as ts-rust's leak_arena.rs drives it: a leaked Bump per
// chunk with its allocation limit set to its size, try_alloc_with that
// fails when full, slice copies, and chunk_capacity checks.
use bumpalo::Bump;

fn new_chunk(capacity: usize) -> &'static Bump {
    let bump = Box::leak(Box::new(Bump::with_capacity(capacity)));
    bump.set_allocation_limit(Some(bump.allocated_bytes()));
    bump
}

fn main() {
    let mut chunk = new_chunk(4096 - 256);
    let mut chunks = 1;
    let mut total = 0u64;
    for i in 0..5000u64 {
        let slot = match chunk.try_alloc_with(|| i * 3) {
            Ok(slot) => slot,
            Err(_) => {
                chunk = new_chunk(4096 - 256);
                chunks += 1;
                chunk.alloc_with(|| i * 3)
            }
        };
        *slot += 1;
        total += *slot;
        if i % 7 == 0 {
            let items = [i as u8; 13];
            let s = match chunk.try_alloc_slice_copy(&items) {
                Ok(s) => s,
                Err(_) => {
                    chunk = new_chunk(4096 - 256);
                    chunks += 1;
                    chunk.alloc_slice_copy(&items)
                }
            };
            total += s[12] as u64;
        }
    }
    println!("{} {} {} {}", chunks, total, chunk.allocated_bytes(), chunk.chunk_capacity());
    let b = Bump::new();
    let v = b.alloc_slice_fill_with(5, |i| i * i);
    println!("{:?} {} {}", v, b.allocated_bytes(), b.chunk_capacity());
    let small = Bump::with_capacity(10);
    small.set_allocation_limit(Some(100));
    println!("{:?} {} {}", small.try_alloc([0u8; 300]).is_err(), small.allocated_bytes(), small.allocation_limit().unwrap());
    let e = bumpalo::AllocErr;
    println!("{} {:?}", e, e);
}
