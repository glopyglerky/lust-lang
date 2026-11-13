// DoubleEndedIterator for Zip (R22): core's TrustedRandomAccess trim of a longer side with side effects, the general trim, drops.
#[derive(Debug, Clone)]
struct C(i32);
impl Drop for C { fn drop(&mut self) { println!("drop C{}", self.0); } }
fn main() {
    let a = vec![1, 2, 3];
    let b = vec![4, 5, 6];
    for (x, y) in a.iter().zip(&b).rev() { print!("{x}{y} "); }
    println!();
    let long = [1, 2, 3, 4, 5];
    let short = [10, 20, 30];
    let mut z = long.iter().map(|x| { println!("map a {x}"); x * 2 }).zip(short.iter().map(|y| { println!("map b {y}"); *y }));
    println!("{:?}", z.next());
    println!("{:?}", z.next_back());
    println!("{:?}", z.size_hint());
    println!("{:?}", z.next_back());
    println!("{:?}", z.next());
    println!("{:?}", z.next_back());
    println!("{:?}", z.next());
    let mut g = vec![C(1), C(2), C(3), C(4)].into_iter().zip(vec![C(10), C(20)]);
    println!("{:?}", g.next_back().map(|p| (p.0.0, p.1.0)));
    println!("{:?}", g.size_hint());
    drop(g);
    let r: Vec<(usize, char)> = (0..10usize).zip("abc".chars().rev().collect::<Vec<_>>()).rev().collect();
    println!("{:?}", r);
    let e: Vec<_> = [1u8, 2, 3].into_iter().zip([4u8, 5]).rev().collect();
    println!("{:?}", e);
    let mut w = (1..=4u8).zip(vec!['x', 'y', 'z'].into_iter().rev());
    println!("{:?} {:?}", w.next_back(), w.next());
    let n: Vec<_> = a.iter().zip(b.iter().zip(long.iter())).rev().map(|(x, (y, z))| x + y + z).collect();
    println!("{:?}", n);
    let cl: Vec<_> = vec![C(7), C(8), C(9)].iter().cloned().zip(0..2).rev().map(|(c, i)| c.0 + i).collect();
    println!("{:?}", cl);
    let mut en = long.iter().copied().zip(short.iter().rev());
    println!("{:?} {:?}", en.next_back(), en.len());
    let last = a.iter().zip(&b).rev().last();
    println!("{:?}", last);
}
