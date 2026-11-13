// Closure call counts per adapter and consumer: Rust 1.99's pull
// order, including Zip's random-access and TrustedLen fold paths.
fn main() {
    let v = vec![1, 2, 3, 4, 5, 6];
    let w = vec![7, 8, 9];
    let mut n = 0;
    let r: Vec<i32> = v.iter().map(|x| { n += 1; *x }).skip(2).take(2).collect();
    println!("skip+take {:?} {}", r, n);
    // Both sides random access: the long side stops at the short one.
    let mut n = 0;
    let z: Vec<(i32, &i32)> = v.iter().map(|x| { n += 1; *x }).zip(w.iter()).collect();
    println!("zip tra {:?} {}", z, n);
    // TrustedLen but not random access: `next` pulls the left once
    // more, `fold` (collect, count, sum) does not.
    let mut n = 0;
    let z: Vec<(i32, &i32)> = v.iter().map(|x| { n += 1; *x }).zip(w.iter().take(3)).collect();
    println!("zip fold {:?} {}", z, n);
    let mut n = 0;
    for p in v.iter().map(|x| { n += 1; *x }).zip(w.iter().take(3)) {
        print!("{:?} ", p);
    }
    println!("zip next {}", n);
    let mut n = 0;
    let c = v.iter().map(|x| { n += 1; *x }).zip(w.iter().take(3)).count();
    println!("zip count {} {}", c, n);
    let mut n = 0;
    let f = v.iter().map(|x| { n += 1; *x }).zip(w.iter().take(3)).filter(|p| p.0 > 1).count();
    println!("zip filter count {} {}", f, n);
    let mut n = 0;
    let any = v.iter().map(|x| { n += 1; *x }).zip(w.iter().take(3)).any(|p| p.0 > 10);
    println!("zip any {} {}", any, n);
    let mut n = 0;
    let found = v.iter().map(|x| { n += 1; *x }).find(|x| *x == 3);
    println!("find {:?} {}", found, n);
    let mut n = 0;
    let s: i32 = v.iter().map(|x| { n += 1; *x }).step_by(2).sum();
    println!("step_by {} {}", s, n);
    let mut n = 0;
    let t: Vec<i32> = v.iter().map(|x| { n += 1; *x }).take_while(|x| *x < 3).collect();
    println!("take_while {:?} {}", t, n);
    let mut n = 0;
    let mut pk = v.iter().map(|x| { n += 1; *x }).peekable();
    pk.peek();
    pk.peek();
    let first = pk.next();
    println!("peek {:?} {}", first, n);
    let mut n = 0;
    let m = v.iter().map(|x| { n += 1; x % 4 }).max();
    println!("max {:?} {}", m, n);
    let mut n = 0;
    let l = v.iter().map(|x| { n += 1; *x }).last();
    println!("last {:?} {}", l, n);
    let (mut n, mut m) = (0, 0);
    let ch: Vec<i32> = v.iter().take(2).map(|x| { n += 1; *x }).chain(w.iter().map(|x| { m += 1; *x })).collect();
    println!("chain {:?} {} {}", ch, n, m);
}
