struct P {
    x: i32,
    y: i32,
    z: i32,
}

struct Q(i32, i32);

fn main() {
    let p = P { x: 1, y: 2, z: 3 };
    let P { x, .. } = p;
    let P { y: yy, z: ref zz, .. } = p;
    println!("{} {} {}", x, yy, zz);
    let q = Q(3, 4);
    let Q(a, b) = q;
    match q {
        Q(0, _) => println!("z"),
        Q(v, ..) => println!("x {}", v),
    }
    println!("{} {}", a, b);
    let arr = [String::from("a"), String::from("b")];
    let [s1, s2] = arr;
    println!("{} {}", s1, s2);
    let arr2 = [String::from("x"), String::from("y"), String::from("z")];
    let [first, .., last] = arr2;
    println!("{} {}", first, last);
    for s in [String::from("m"), String::from("n")] {
        println!("{}", s);
    }
    let t = (String::from("t"), 7, [String::from("u")]);
    let s = t.0;
    let n = t.1;
    let [u] = t.2;
    println!("{} {} {} {}", s, n, u, t.1);
    let mut a1 = 1;
    let mut b1 = 2;
    {
        let t2 = (&mut a1, &mut b1);
        *t2.0 += 5;
        *t2.1 *= 3;
    }
    println!("{} {}", a1, b1);
    let mut t3 = (1, String::from("a"));
    {
        let (ref mut n2, ref mut s3) = t3;
        *n2 += 10;
        s3.push('b');
    }
    println!("{} {}", t3.0, t3.1);
}
