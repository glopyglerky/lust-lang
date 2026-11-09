// Places reached through a subslice temporary: `&v[a..][..b]`, `&v[a..][i]`, `&mut` forms, str
// (ts-rust fast_mode.rs cost_split).
use std::cmp::Reverse;
fn dep_share() -> u64 { 50 }
pub fn cost_split(costs: &[u64], weights: &[u64], imports: &[Vec<usize>], count: usize) -> Vec<usize> {
    let files = costs.len();
    let words = files.div_ceil(64);
    let mut read = vec![0u64; count * words];
    let mut load = vec![0u64; count];
    let mut checker_of = vec![usize::MAX; files];
    let mut order: Vec<usize> = (0..files).filter(|&f| costs[f] > 0).collect();
    order.sort_by_key(|&f| (Reverse(costs[f]), f));
    let share = dep_share();
    for f in order {
        let mut best = (u64::MAX, 0);
        for (checker, &checker_load) in load.iter().enumerate() {
            let row = &read[checker * words..][..words];
            let new_reads: u64 = imports[f].iter().filter(|&&d| row[d / 64] >> (d % 64) & 1 == 0).map(|&d| weights[d]).sum();
            let after = checker_load + costs[f] + new_reads * share / 100;
            if after < best.0 {
                best = (after, checker);
            }
        }
        let (after, checker) = best;
        load[checker] = after;
        checker_of[f] = checker;
        let row = &mut read[checker * words..][..words];
        for &d in imports[f].iter().chain(std::iter::once(&f)) {
            row[d / 64] |= 1 << (d % 64);
        }
    }
    let mut next = 0;
    for checker in checker_of.iter_mut().filter(|checker| **checker == usize::MAX) {
        *checker = next % count;
        next += 1;
    }
    checker_of
}
fn main_cost() {
    let v = cost_split(&[5, 0, 3, 7], &[1, 2, 3, 4], &[vec![1], vec![], vec![0, 1], vec![2]], 2);
    println!("{:?}", v);
}
fn tail(v: &[u8]) -> &[u8] {
    &v[1..][..2]
}
fn main() {
    main_cost();
    let mut v = vec![1u8, 2, 3, 4, 5];
    let a = &v[1..][..2];
    let b = &v[1..][1];
    println!("{:?} {}", a, b);
    let m = &mut v[1..][..3];
    m[0] = 9;
    let e = &mut v[2..][1];
    *e += 1;
    println!("{:?} {:?}", v, tail(&v));
    let s = String::from("hello");
    println!("{}", &s[1..][..3]);
}
