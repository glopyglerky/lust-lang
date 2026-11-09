// Copies whose length is known only at run time, through the runtime's type-erased paths
// (Vec insert and remove, HashMap growth, the stable and unstable sorts, Vec clone, Vec and
// String drain, String push), for element sizes on every side of the inline small copy's
// cases: 0, 1-3, 4-7, 8-16, 17-32 and over 32 bytes.
use std::collections::HashMap;

fn elem<const N: usize>(i: usize) -> [u8; N] {
    let mut a = [0u8; N];
    for (j, b) in a.iter_mut().enumerate() {
        *b = (i * 31 + j * 7 + 1) as u8;
    }
    a
}

fn mix<const N: usize>(v: &[[u8; N]]) -> u64 {
    v.iter().flatten().fold(17u64, |h, &b| h.wrapping_mul(131).wrapping_add(b as u64))
}

fn run<const N: usize>() {
    // Vec insert and remove at every position of a short Vec.
    let mut v: Vec<[u8; N]> = (0..6).map(elem::<N>).collect();
    for i in 0..=v.len() {
        v.insert(i, elem::<N>(100 + i));
    }
    for i in (0..v.len()).step_by(3) {
        if i < v.len() {
            v.remove(i);
        }
    }
    let a = mix(&v);
    // HashMap growth: every element moves at each resize.
    let mut m: HashMap<u32, [u8; N]> = HashMap::new();
    for i in 0..200u32 {
        m.insert(i, elem::<N>(i as usize));
    }
    let mut keys: Vec<u32> = m.keys().copied().collect();
    keys.sort();
    let b = keys.iter().map(|k| mix(&[m[k]])).fold(0u64, |x, y| x.wrapping_mul(7).wrapping_add(y));
    // Sorts, stable and unstable, with duplicates, long enough for the merge and quick paths.
    let mut s: Vec<[u8; N]> = (0..300).map(|i| elem::<N>((i * 7919) % 97)).collect();
    let mut u = s.clone();
    s.sort();
    u.sort_unstable_by(|x, y| y.cmp(x));
    let c = mix(&s) ^ mix(&u).rotate_left(1);
    // Vec clone of short lengths, and drains that leave tails of every length.
    let mut d = 0u64;
    for len in 0..6 {
        let w: Vec<[u8; N]> = (0..len).map(elem::<N>).collect();
        let w2 = w.clone();
        d = d.wrapping_mul(3).wrapping_add(mix(&w2));
        for start in 0..=len {
            for end in start..=len {
                let mut x = w.clone();
                let gone: Vec<[u8; N]> = x.drain(start..end).collect();
                d = d.wrapping_mul(5).wrapping_add(mix(&x) ^ mix(&gone));
            }
        }
    }
    println!("{N:2}: {a:x} {b:x} {c:x} {d:x}");
}

fn main() {
    run::<0>();
    run::<1>();
    run::<2>();
    run::<3>();
    run::<4>();
    run::<5>();
    run::<7>();
    run::<8>();
    run::<9>();
    run::<12>();
    run::<15>();
    run::<16>();
    run::<17>();
    run::<24>();
    run::<31>();
    run::<32>();
    run::<33>();
    run::<40>();
    // String push and drain of every short length.
    let mut s = String::new();
    for n in 0..40 {
        let piece: String = (0..n).map(|i| (b'a' + ((i + n) % 26) as u8) as char).collect();
        s.push_str(&piece);
    }
    let mut t = s.clone();
    let mid: String = t.drain(5..300).collect();
    println!("str {} {} {} {}", s.len(), &s[s.len() - 12..], t.len(), &mid[..20]);
}
