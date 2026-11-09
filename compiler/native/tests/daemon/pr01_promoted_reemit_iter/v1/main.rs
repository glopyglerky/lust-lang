// Promoted refs in fns with iterator builtins (next, sum, a for loop).
// Only fill.rs changes between versions; it is declared first, so the
// fns after it move slots when its dead fillers compact away, and a
// field edit puts the request in full mode. Those fns re-emit C from
// cached IR (the vN stats file asks for reemitted), and every promoted temp must keep
// static storage: the refs are read after a stack-clobbering call.
mod fill;
mod a;

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
    let p = a::pair(vec![1, 2]);
    let q = a::sums(vec![3, 4]);
    let r = a::forl(vec![5]);
    let k = clobber();
    println!("{p:?} {q:?} {r:?} {k} {}", fill::mk());
}
