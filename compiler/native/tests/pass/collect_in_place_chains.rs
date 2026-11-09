// In-place collect through adapter chains; Copied, StepBy, Rev, and
// FlatMap take the fresh-allocation path (as in Rust 1.99).
fn mk() -> Vec<u64> {
    let mut v: Vec<u64> = Vec::with_capacity(16);
    v.extend([1, 2, 3, 4, 5]);
    v
}
fn main() {
    let fm: Vec<u64> = mk().into_iter().flat_map(|x| [x, x]).collect();
    let fm1: Vec<u64> = mk().into_iter().flat_map(|x| [x]).collect();
    let (t3, t4) = (3u64, 4u64);
    let src: Vec<&u64> = { let mut v = Vec::with_capacity(16); v.push(&t3); v.push(&t4); v };
    let cp: Vec<u64> = src.clone().into_iter().copied().collect();
    let cl: Vec<u64> = src.into_iter().cloned().collect();
    let multi: Vec<u64> = mk().into_iter().map(|x| x + 1).filter(|x| x % 2 == 0).skip(1).take(5).enumerate().map(|(i, x)| x + i as u64).collect();
    let fmv: Vec<u64> = mk().into_iter().flat_map(|x| vec![x]).collect();
    let chunks: Vec<u64> = mk().into_iter().step_by(2).collect();
    let rv: Vec<u64> = mk().into_iter().rev().map(|x| x).collect();
    let zz: Vec<(u64, u64)> = mk().into_iter().zip(vec![1u64; 3]).collect();
    println!("fm {} fm1 {} cp {} cl {} multi {} {:?} fmv {} chunks {} rv {} zz {}", fm.capacity(), fm1.capacity(), cp.capacity(), cl.capacity(), multi.capacity(), multi, fmv.capacity(), chunks.capacity(), rv.capacity(), zz.capacity());
}
