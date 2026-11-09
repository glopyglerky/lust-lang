// 400 seeded random vectors: sort_by and sort_unstable_by comparator
// call counts, the exact result order (stable and unstable), and
// binary_search_by, as Rust's driftsort and ipnsort give them.
fn next(s: &mut u64) -> u64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}
fn main() {
    let mut seed: u64 = 0x853c49e6748fea9b;
    let mut total_calls: u64 = 0;
    let mut check: u64 = 0;
    for round in 0..400u64 {
        let n = (next(&mut seed) % 120) as usize;
        let range = 1 + next(&mut seed) % 40;
        let mut v: Vec<(u64, u64)> = Vec::new();
        for i in 0..n {
            v.push((next(&mut seed) % range, i as u64));
        }
        let mut a = v.clone();
        let mut calls = 0u64;
        a.sort_by(|x, y| {
            calls += 1;
            x.0.cmp(&y.0)
        });
        let mut b = v.clone();
        let mut calls_u = 0u64;
        b.sort_unstable_by(|x, y| {
            calls_u += 1;
            x.0.cmp(&y.0)
        });
        let mut c: Vec<u64> = v.iter().map(|p| p.0 * 1000 + p.1).collect();
        c.sort_unstable();
        let probe = next(&mut seed) % (range + 2);
        let found = match c.binary_search_by(|z| (z / 1000).cmp(&probe)) {
            Ok(_) => 1,
            Err(i) => 2 + i as u64,
        };
        total_calls += calls * 3 + calls_u;
        for (k, p) in a.iter().enumerate() {
            check = check.wrapping_mul(31).wrapping_add(p.1 * (k as u64 + 1));
        }
        for p in b.iter() {
            check = check.wrapping_mul(7).wrapping_add(p.0 * 131 + p.1);
        }
        check = check.wrapping_add(found * (round + 1));
        if round % 50 == 0 {
            println!("{} {} {} {} {}", round, n, calls, calls_u, found);
        }
    }
    println!("{} {}", total_calls, check);
}
