enum Shape {
    Pair(i64, i64),
    One(i64),
}

struct Rec {
    a: i64,
    b: i64,
}

fn main() {
    let s = Shape::Pair(3, 4);
    match &s {
        Shape::Pair(a, b) => println!("{} {}", a, b),
        Shape::One(a) => println!("{}", a),
    }
    match s {
        whole @ Shape::Pair(a, _) => match whole {
            Shape::Pair(x, y) => println!("{} {} {}", a, x, y),
            Shape::One(_) => println!("no"),
        },
        Shape::One(_) => println!("one"),
    }

    let p = (1, 2);
    match &p {
        &(x, y) => println!("{} {}", x, y),
    }

    let mut m = 5;
    match &mut m {
        &mut v => println!("{}", v),
    }

    let r = Rec { a: 7, b: 8 };
    match r {
        Rec { a: v @ 7, b } => println!("{} {}", v, b),
        _ => println!("other"),
    }

    let n = 12;
    match n {
        v @ 10..=20 => println!("in {}", v),
        _ => println!("out"),
    }

    let o = Some(9);
    match &o {
        &Some(v) => println!("{}", v),
        &None => println!("none"),
    }
}
