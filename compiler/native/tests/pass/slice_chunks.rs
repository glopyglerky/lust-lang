// `chunks`, `chunks_mut`, `chunks_exact`, and `chunks_exact_mut`
// (tf-slice): front and back pulls, the exact size, the
// remainder, and adapter chains as in ts-rust's uses.
fn main() {
    let v = vec![1, 2, 3, 4, 5];
    for c in v.chunks(2) {
        print!("{:?} ", c);
    }
    println!();
    for c in v.chunks(2).rev() {
        print!("{:?} ", c);
    }
    println!();
    println!("{} {:?}", v.chunks(2).len(), v.chunks(2).size_hint());
    println!("{:?}", v.chunks(3).last());
    println!("{}", v.chunks_exact(2).any(|p| p[0] == 3));
    let pairs: Vec<(i32, i32)> = v.chunks_exact(2).map(|p| (p[0], p[1])).collect();
    println!("{:?}", pairs);
    let mut it = v.chunks_exact(2);
    println!("{:?}", it.remainder());
    println!("{:?}", it.next());
    println!("{:?} {:?}", it.remainder(), it.next_back());
    println!("{:?} {}", it.next(), it.len());
    let a = [1u8, 2, 3, 4, 5, 6, 7, 8];
    let b = [1u8, 2, 3, 4, 9, 6, 7, 8];
    for (x, y) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
        println!("{}", x[0] ^ y[0] + x[3] ^ y[3]);
    }
    let mut m = vec![0u8; 8];
    for (i, c) in m.chunks_exact_mut(4).enumerate() {
        c.copy_from_slice(&[i as u8, 0, 0, 0]);
    }
    println!("{:?}", m);
    let mut m2 = vec![1, 2, 3, 4, 5];
    for c in m2.chunks_mut(2) {
        c.fill(9);
    }
    println!("{:?}", m2);
    let mut m3 = vec![1, 2, 3, 4, 5];
    let it3 = m3.chunks_exact_mut(2);
    println!("{:?}", it3.into_remainder());
    let ints: Vec<u16> = b"abcd".chunks_exact(2).map(|p| p[0] as u16 * 256 + p[1] as u16).collect();
    println!("{:?}", ints);
    println!("{} {:?}", v.chunks(9).count(), v.chunks_exact(9).next());
    let e: Vec<i32> = vec![];
    println!("{:?} {:?}", e.chunks(2).next(), e.chunks_exact(2).remainder());
}
