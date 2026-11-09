// [T]::as_chunks, as_rchunks and their _mut forms (R13): literal, inferred, and generic N.
fn f120(src: &[u8], dst: &mut Vec<u8>) -> u32 {
    let (whole, _rest) = src.as_chunks::<3>();
    let (quads, _) = dst.as_chunks_mut::<4>();
    let mut t = 0;
    for (quad, &[b0, b1, b2]) in quads.iter_mut().zip(whole) { quad[0] = b0; t += (b1 + b2) as u32; }
    t
}
fn sum<const N: usize>(s: &[u32]) -> (usize, usize) { let (a, b) = s.as_chunks::<N>(); (a.len(), b.len()) }
fn main() {
    let src: &[u8] = b"abcdefg";
    let (whole, rest) = src.as_chunks::<3>();
    println!("{:?} {:?}", whole, rest);
    let (r, c) = src.as_rchunks::<2>();
    println!("{:?} {:?}", r, c);
    let mut dst = vec![0u8; 9];
    let (quads, tail) = dst.as_chunks_mut::<4>();
    quads[0][0] = 1;
    quads[1][3] = 2;
    tail[0] = 3;
    println!("{:?}", dst);
    let (t, q) = dst.as_rchunks_mut::<2>();
    t[0] = 7;
    q[3][1] = 8;
    println!("{:?}", dst);
    let arr = [1u16, 2, 3, 4, 5];
    let (pairs, odd): (&[[u16; 2]], &[u16]) = arr.as_chunks();
    println!("{:?} {:?}", pairs, odd);
    println!("{}", f120(&[1, 2, 3, 4, 5, 6], &mut vec![0; 8]));
    println!("{:?} {:?}", sum::<1>(&[1, 2, 3]), sum::<5>(&[1, 2, 3]));
    let e: &[String] = &[];
    let (a, b) = e.as_chunks::<2>();
    println!("{} {}", a.len(), b.len());
    let words = vec![String::from("a"), String::from("b"), String::from("c")];
    let (w2, w1) = words.as_chunks::<2>();
    println!("{:?} {:?}", w2, w1);
}
