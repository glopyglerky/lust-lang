// Iterator::size_hint (W-STD-C phase 2): concrete adapters at
// several points of consumption, and through a generic bound.
fn generic<I: Iterator>(i: &I) -> (usize, Option<usize>) {
    i.size_hint()
}

fn main() {
    let v = vec![10, 20, 30, 40];
    let mut a = v.iter();
    println!("{:?} {:?}", a.size_hint(), generic(&a));
    a.next();
    println!("{:?}", a.size_hint());
    let b = (0..10).step_by(3);
    println!("{:?}", b.size_hint());
    let c = (0..10).filter(|x| x % 2 == 0);
    println!("{:?}", c.size_hint());
    let d = v.iter().copied().chain(0..2);
    println!("{:?}", d.size_hint());
    let e = (0..5).zip(0..9);
    println!("{:?}", e.size_hint());
    let f = (0..9).skip(4).take(3);
    println!("{:?}", f.size_hint());
    let g = "héllo".chars();
    println!("{:?}", g.size_hint());
    let h = (0..6).rev().enumerate();
    println!("{:?}", h.size_hint());
    let i = (0..6).peekable();
    println!("{:?}", i.size_hint());
    let j: Vec<i32> = (0..4).collect();
    let k = j.iter();
    println!("{:?}", k.size_hint());
}
