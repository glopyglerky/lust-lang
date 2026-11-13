fn describe(s: &[i64]) {
    match s {
        [] => println!("empty"),
        [x] => println!("one {}", x),
        [x, y] => println!("two {} {}", x, y),
        [first, middle @ .., last] => println!("many {} {} {:?}", first, last, middle),
    }
}

fn main() {
    describe(&[]);
    describe(&[5]);
    describe(&[5, 6]);
    describe(&[5, 6, 7, 8]);

    let a = [1, 2, 3, 4];
    match a {
        [head, tail @ ..] => println!("{} {:?}", head, tail),
    }
    let [first, .., last] = a;
    println!("{} {}", first, last);

    match a {
        [1, x, ..] => println!("starts {}", x),
        _ => println!("other"),
    }

    let s: &[i64] = &[10, 20, 30];
    match s {
        [.., z] => println!("{}", z),
        [] => println!("none"),
    }

    let n = 7;
    match n {
        ..0 => println!("neg"),
        0..10 => println!("small"),
        10.. => println!("big"),
        _ => println!("never"),
    }
    match n {
        ..=6 => println!("low"),
        _ => println!("high"),
    }
}
