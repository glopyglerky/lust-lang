// Promoted refs with no expansion: nested, zero-sized, const refs, enum values.
// Only fill.rs changes between versions; it is declared first, so the
// fns after it move slots when its dead fillers compact away, and a
// field edit puts the request in full mode. Those fns re-emit C from
// cached IR (the vN stats file asks for reemitted), and every promoted temp must keep
// static storage: the refs are read after a stack-clobbering call.
mod fill;
mod a;
mod b;

#[inline(never)]
fn clobber() -> u64 {
    let mut a = [0x5au8; 512];
    let mut i = 0;
    while i < 512 {
        a[i] = (i * 7) as u8 ^ 0xa5;
        i += 1;
    }
    let mut s = 0u64;
    for x in a.iter() {
        s += *x as u64;
    }
    s
}

fn main() {
    let n = a::nested();
    let z = a::zst();
    let c = a::konst();
    let u = a::unit_variant();
    let k = clobber();
    println!("{n:?} {z:?} {c:?} {u:?} {k} {}", fill::mk());
}
