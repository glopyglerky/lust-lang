// `rev` over `Enumerate` and `Skip` (ts-rust store.rs `mark_roots`:
// `iter().zip(..).enumerate().skip(1).rev()`): core's `next_back` index is
// `count + inner.len()`, and `Skip` pulls from the back while `len() > 0`.
fn main() {
    let records = vec![10u8, 20, 30, 40];
    let kinds = vec![1u16, 2, 3];
    let slots = records.iter().zip(&kinds).enumerate().skip(1).rev();
    for (start, (r, &k)) in slots { println!("{start} {r} {k}"); }
    let v = vec!['a', 'b', 'c', 'd', 'e'];
    println!("{:?}", v.iter().enumerate().rev().collect::<Vec<_>>());
    println!("{:?}", v.iter().skip(2).rev().collect::<Vec<_>>());
    println!("{:?}", v.iter().skip(9).rev().collect::<Vec<_>>());
    println!("{:?}", v.iter().enumerate().skip(1).rev().nth(1));
    println!("{:?}", v.iter().skip(1).enumerate().rev().last());
    println!("{:?}", v.iter().enumerate().rev().fold(0, |a, (i, _)| a * 10 + i));
    let mut it = v.iter().enumerate().skip(1);
    println!("{:?} {:?} {:?}", it.next(), it.next_back(), it.len());
    println!("{:?} {:?}", it.next_back(), it.next());
    println!("{:?}", it.collect::<Vec<_>>());
    let mut e = (0..6u32).enumerate();
    e.next();
    println!("{:?} {:?}", e.next_back(), e.len());
    println!("{:?}", v.iter().map(|c| { print!("m{c} "); *c }).enumerate().skip(2).rev().take(2).collect::<Vec<_>>());
    println!("{:?}", (1..=4u8).skip(1).rev().map(|x| x * 2).collect::<Vec<_>>());
    println!("{:?}", v.iter().enumerate().rev().position(|(_, c)| *c == 'b'));
    println!("{:?}", v.into_iter().enumerate().skip(1).rev().collect::<Vec<_>>());
}
