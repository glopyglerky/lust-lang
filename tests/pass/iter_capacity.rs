// `collect` capacity: the exact upper bound for TrustedLen chains,
// the first-item rule then `reserve(lower + 1)` for the rest, and the
// `String` impls per item type (W-STD-A phase 3).
fn main() {
    let v: Vec<u64> = (0..37).collect();
    let a: Vec<u64> = v.iter().map(|x| x * 2).collect();
    let b: Vec<&u64> = v.iter().filter(|x| **x % 3 == 0).collect();
    let c: Vec<u64> = v.iter().copied().skip(5).collect();
    let d: Vec<(usize, &u64)> = v.iter().enumerate().take(9).collect();
    let e: Vec<u64> = v.iter().copied().chain(0..4).collect();
    let f: Vec<u8> = (0..100u8).filter(|x| x % 7 == 0).collect();
    let g: Vec<u64> = v.iter().copied().step_by(5).collect();
    let h: Vec<u64> = v.iter().copied().take_while(|x| *x < 11).collect();
    let i: Vec<char> = "héllo wörld".chars().collect();
    let j: Vec<u64> = Vec::new().into_iter().collect();
    let k: Vec<[u8; 2000]> = (0..3).map(|_| [0u8; 2000]).filter(|_| true).collect();
    println!("{} {} {} {} {} {} {} {} {} {} {}", a.capacity(), b.capacity(), c.capacity(), d.capacity(), e.capacity(), f.capacity(), g.capacity(), h.capacity(), i.capacity(), j.capacity(), k.capacity());
    let s1: String = "abcdefgh".chars().rev().collect::<Vec<char>>().into_iter().collect();
    let s2: String = "ab".chars().chain("cdefghijk".chars()).collect();
    let s3: String = vec!["x", "yz", "w"].into_iter().collect();
    let s4: String = vec![String::with_capacity(50), String::from("tail")].into_iter().collect();
    let s5: String = "words here".chars().filter(|c| *c != ' ').collect();
    println!("{} {} {} {} {}", s1.capacity(), s2.capacity(), s3.capacity(), s4.capacity(), s5.capacity());
    println!("{} {} {} {} {}", s1, s2, s3, s4, s5);
}
