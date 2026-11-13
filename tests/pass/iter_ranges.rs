// Integer ranges as iterator values: rev, inclusive ends, and the
// full-width bounds (W-STD-A phase 3).
fn main() {
    let r = 2..6;
    println!("{:?}", r.map(|x| x * x).collect::<Vec<i32>>());
    for i in (0..5).rev() {
        print!("{} ", i);
    }
    println!();
    println!("{:?}", (250u8..=255).collect::<Vec<u8>>());
    println!("{:?}", (250u8..=255).rev().collect::<Vec<u8>>());
    println!("{:?}", (i8::MIN..=i8::MIN + 2).collect::<Vec<i8>>());
    println!("{} {}", (0..0).count(), (5..=4).count());
    println!("{:?}", (u64::MAX - 2..=u64::MAX).collect::<Vec<u64>>());
    let s: i32 = (1..=100).sum();
    println!("{}", s);
    let evens: Vec<usize> = (0..10).filter(|x| x % 2 == 0).collect();
    println!("{:?} {}", evens, evens.capacity());
    let mut it = 1..=3;
    println!("{:?} {:?} {:?} {:?}", it.next(), it.next(), it.next(), it.next());
    let zipped: Vec<(i32, char)> = (0..3).zip("abc".chars()).collect();
    println!("{:?}", zipped);
}
