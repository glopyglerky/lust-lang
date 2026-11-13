// The comparator call sequence is observable; lust's port of driftsort
// and ipnsort makes the same calls in the same order as Rust 1.99, so
// call counts and a running checksum of the compared pairs match.
fn main() {
    for &n in &[5usize, 20, 21, 40, 150, 700] {
        let mut v: Vec<u32> = (0..n as u32).map(|i| (i * 37 + 11) % 23).collect();
        let (mut calls, mut sum) = (0u64, 0u64);
        v.sort_by(|a, b| {
            calls += 1;
            sum = sum.wrapping_mul(31).wrapping_add((*a as u64) * 7 + *b as u64);
            a.cmp(b)
        });
        let mut w: Vec<u32> = (0..n as u32).map(|i| (i * 37 + 11) % 23).collect();
        let (mut calls2, mut sum2) = (0u64, 0u64);
        w.sort_unstable_by(|a, b| {
            calls2 += 1;
            sum2 = sum2.wrapping_mul(31).wrapping_add((*a as u64) * 7 + *b as u64);
            b.cmp(a)
        });
        let mut keys = 0u64;
        let mut k: Vec<String> = (0..n).map(|i| format!("{}", (i * 13) % 17)).collect();
        k.sort_by_key(|s| {
            keys += 1;
            s.clone()
        });
        println!("{} {} {} {} {} {} {:?}", n, calls, sum, calls2, sum2, keys, &k[..3]);
    }
}
