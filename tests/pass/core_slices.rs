fn sum(s: &[i64]) -> i64 {
    let mut t = 0;
    let mut i = 0usize;
    while i < s.len() {
        t += s[i];
        i += 1;
    }
    t
}

fn bump(s: &mut [i64]) {
    let mut i = 0usize;
    while i < s.len() {
        s[i] += 1;
        i += 1;
    }
}

fn main() {
    let a = [1, 2, 3, 4, 5];
    println!("{}", sum(&a));
    println!("{}", sum(&a[1..4]));
    println!("{}", sum(&a[..2]));
    println!("{}", sum(&a[3..]));
    println!("{}", sum(&a[..]));
    println!("{}", sum(&a[1..=2]));

    let v = vec![10, 20, 30];
    println!("{}", sum(&v));
    println!("{}", sum(&v[1..]));

    let s: &[i64] = &a;
    println!("{} {} {:?}", s.len(), s.is_empty(), s);

    let empty: &[i64] = &a[0..0];
    println!("{} {}", empty.len(), empty.is_empty());

    println!("{}", a.len());
    println!("{}", a[1..3].len());
    println!("{:?}", &a[2..5]);

    let mut m = [1, 2, 3];
    bump(&mut m);
    println!("{:?}", m);
    bump(&mut m[1..]);
    println!("{:?}", m);

    let mut vm = vec![5, 6, 7];
    bump(&mut vm);
    println!("{:?}", vm);

    for x in &a {
        print!("{} ", x);
    }
    println!();
    for x in a {
        print!("{} ", x);
    }
    println!();
    let sl: &[i64] = &[7, 8];
    for x in sl {
        print!("{} ", x);
    }
    println!();
    for x in &m {
        print!("{} ", x);
    }
    println!();

    println!("{}", &a[..2] == &[1, 2][..]);
}
