// Zip::next_back over two slice sides reads each side by index (lower/iter.rs zip_next_back): unequal
// lengths, mixed ends, an enumerate/skip/rev chain, mutable sides, and a side with side effects.
fn main() {
    let a = [1u32, 2, 3, 4, 5];
    let b = ["x", "y", "z"];
    // Unequal lengths, back to front.
    let v: Vec<(u32, &str)> = a.iter().zip(b.iter()).rev().map(|(x, y)| (*x, *y)).collect();
    println!("{v:?}");
    // Mixed ends.
    let mut z = a.iter().zip(a.iter().skip(1));
    println!("{:?} {:?} {:?} {:?}", z.next(), z.next_back(), z.next(), z.next_back());
    println!("{:?} {:?}", z.next(), z.next_back());
    // mark_roots' shape.
    let kinds = [7u16, 8, 9, 10];
    for (i, (r, &k)) in a.iter().zip(&kinds).enumerate().skip(1).rev() {
        print!("{i}:{r}:{k} ");
    }
    println!();
    // Mutable sides.
    let mut m = [10i64, 20, 30];
    let mut n = [1i64, 2, 3, 4];
    for (x, y) in m.iter_mut().zip(n.iter_mut()).rev() {
        *x += *y;
        *y *= 2;
    }
    println!("{m:?} {n:?}");
    // A side with side effects keeps the general form.
    let mut calls = 0;
    let w: Vec<u32> = a.iter().map(|x| { calls += 1; x * 10 }).zip(b.iter()).rev().map(|(x, _)| x).collect();
    println!("{w:?} {calls}");
    // Empty and len after next_back.
    let e: [u8; 0] = [];
    println!("{:?}", e.iter().zip(a.iter()).rev().next());
    // Taken from both ends, then folded (collect, and a `for` loop).
    let mut p = a.iter().zip(kinds.iter());
    p.next();
    p.next_back();
    let rest: Vec<(u32, u16)> = p.map(|(x, y)| (*x, *y)).collect();
    println!("{rest:?}");
    let mut p2 = a.iter().zip(a.iter().skip(2));
    p2.next_back();
    p2.next();
    let mut s = 0u32;
    for (x, y) in p2 {
        s = s * 7 + x * y;
    }
    println!("{s}");
    let mut q = a.iter().zip(b.iter());
    q.next_back();
    println!("{} {:?}", q.len(), q.collect::<Vec<_>>());
}
