// `slice::windows` (W-STD-B): forward and back, as a TrustedLen and
// random-access iterator (zip pulls only as many items from a mapped
// side as the windows give), with std's exact size.
fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn main() {
    let v = [1, 2, 3, 4];
    for w in v.windows(2) {
        println!("{w:?}");
    }
    let all: Vec<&[i32]> = v.windows(3).collect();
    println!("{all:?} {}", all.capacity());
    println!("{:?}", v.windows(2).rev().collect::<Vec<_>>());
    println!("{} {}", v.windows(5).count(), v.windows(4).count());
    println!("{:?} {:?}", find(b"hello world", b"o w"), find(b"abc", b"x"));
    let mut hits = 0;
    let w = [10, 20, 30, 40, 50, 60];
    let z: Vec<_> = w.iter().map(|x| {
        hits += 1;
        x * 2
    }).zip(v.windows(2)).collect();
    println!("{} {hits} {:?}", z.len(), z[0]);
    let vec = vec![5u8, 6, 7];
    let sums: Vec<u32> = vec.windows(2).map(|p| p[0] as u32 + p[1] as u32).collect();
    println!("{sums:?} {:?}", vec.windows(2).last());
    let s: &[u8] = &vec;
    println!("{:?} {:?}", s.windows(1).nth(2), s.windows(2).rposition(|p| p[0] == 5));
    let mut it = v.windows(2);
    println!("{:?} {:?} {:?}", it.next(), it.next(), it.next());
}
