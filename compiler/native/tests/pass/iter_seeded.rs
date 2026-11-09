// 300 seeded random inputs through fixed iterator chains: adapters,
// consumers, peekable, and inspect pull counts, as Rust computes them.
fn next(s: &mut u64) -> u64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}
fn main() {
    let mut seed: u64 = 0x6a09e667f3bcc908;
    let mut acc: u64 = 0;
    for round in 0..300u64 {
        let n = (next(&mut seed) % 40) as usize;
        let v: Vec<u64> = (0..n).map(|_| next(&mut seed) % 50).collect();
        let w: Vec<u64> = (0..(next(&mut seed) % 30)).map(|i| i * 3).collect();
        let k = (next(&mut seed) % 10) as usize;
        let t = next(&mut seed) % 50;
        let mut pulls = 0u64;
        let s1: u64 = v.iter().inspect(|_| pulls += 1).map(|x| x * 2).filter(|x| x % 3 != 0).take(k).sum();
        let mut pulls2 = 0u64;
        let c2 = v.iter().zip(w.iter()).inspect(|_| pulls2 += 1).skip(k / 2).count();
        let pos = v.iter().position(|&x| x > t);
        let any = v.iter().rev().any(|&x| x == t);
        let all = v.iter().all(|&x| x < 49);
        let mx = match v.iter().enumerate().max_by_key(|p| *p.1) {
            Some(p) => Some(p.0),
            None => None,
        };
        let mn = v.iter().copied().min();
        let ch: Vec<u64> = v.iter().chain(w.iter()).step_by(k + 1).copied().collect();
        let mut pk = v.iter().peekable();
        let mut runs = 0u64;
        while let Some(x) = pk.next() {
            if let Some(y) = pk.peek() {
                if *y > x {
                    runs += 1;
                }
            }
        }
        let fm: u64 = v.iter().filter_map(|&x| if x % 2 == 0 { Some(x / 2) } else { None }).fold(1u64, |a, b| a.wrapping_mul(b + 1) % 1000003);
        let tw: Vec<u64> = v.iter().copied().take_while(|&x| x != t).skip_while(|&x| x < 10).collect();
        let last = v.iter().rev().skip(k).last();
        acc = acc.wrapping_mul(1099511628211).wrapping_add(s1 ^ pulls ^ (c2 as u64) << 3 ^ pulls2 << 7);
        let pv = match pos {
            Some(p) => p as u64,
            None => 99,
        };
        acc = acc.wrapping_add(pv).wrapping_add(any as u64 * 3 + all as u64 * 5);
        acc = acc.wrapping_add(mx.unwrap_or(77) as u64 * 11 + mn.unwrap_or(66) * 13 + runs * 17 + fm);
        acc = acc.wrapping_add(ch.iter().sum::<u64>() * 19 + tw.len() as u64 * 23 + (match last {
            Some(x) => *x,
            None => 55,
        }) * 29);
        if round % 60 == 0 {
            println!("{} {} {} {} {} {:?} {:?} {}", round, s1, pulls, c2, pulls2, pos, ch.len(), fm);
        }
    }
    println!("{}", acc);
}
