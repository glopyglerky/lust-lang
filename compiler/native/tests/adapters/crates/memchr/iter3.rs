// memchr 2.8.2: two- and three-needle searches and iterators, forward,
// reverse and mixed from both ends, on empty, short and long haystacks
// (the adapter scans `start..end` sub-slices).
use memchr::{memchr2, memchr2_iter, memchr3, memchr3_iter, memrchr2, memrchr3};

fn main() {
    let long: Vec<u8> = (0..5000u32).map(|i| (i * 31 % 251) as u8).collect();
    for h in [&b""[..], b"x", b"abcabc", b"zzzzzzzzzzzzzzzzzzzzza", &long[..]] {
        println!("{:?} {:?} {:?} {:?}", memchr2(b'a', b'c', h), memrchr2(b'a', b'c', h), memchr3(b'b', b'c', 7, h), memrchr3(b'b', b'c', 7, h));
        let f: Vec<usize> = memchr3_iter(b'a', b'b', 250, h).collect();
        let r: Vec<usize> = memchr3_iter(b'a', b'b', 250, h).rev().collect();
        println!("  {} {:?} {:?} {}", f.len(), f.iter().take(4).collect::<Vec<_>>(), r.iter().take(4).collect::<Vec<_>>(), r.iter().rev().eq(f.iter()));
        let mut it = memchr2_iter(b'a', 7, h);
        let mut both = Vec::new();
        loop {
            match (it.next(), it.next_back()) {
                (None, None) => break,
                (a, b) => both.push((a, b)),
            }
        }
        println!("  ends {:?}", &both[..both.len().min(3)]);
    }
}
