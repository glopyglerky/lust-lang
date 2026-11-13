// RangeInclusive is ExactSizeIterator over u8, i8, u16, and i16 only
// (core's range_incl_exact_iter_impl!), also through adapters and a
// later use that fixes the integer type.
fn f<I: ExactSizeIterator>(i: I) -> usize {
    i.len()
}

fn main() {
    println!("{} {} {}", (0u8..=5).len(), (5i8..=-3).len(), (-128i8..=127).len());
    println!("{}", (0u16..=65535).rev().map(|x| x).len());
    let mut r = 250u8..=255;
    r.next();
    println!("{} {:?}", r.len(), r.size_hint());
    for _ in 0..5 {
        r.next();
    }
    println!("{}", r.len());
    let r = 0..=5;
    let n: u16 = *r.end();
    println!("{}", r.len() + n as usize);
    println!("{} {}", f(0i16..=-1), f((1u8..=3).enumerate()));
    let q = 1..=3;
    println!("{}", q.len());
    let _s: i8 = *q.start();
}
