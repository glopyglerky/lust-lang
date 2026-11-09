// Drop flags on every kind of path (static drop flags, `ir::dropflags`): a local moved on some
// paths only drops where it is still owned; one moved on every path never drops; one moved on
// none always drops. Moves in if/else and match arms, a loop that moves and reinitializes, a
// value made fresh per iteration and moved on some, partial moves of struct fields with the
// field reinitialized, reassignment of a maybe-moved local (the old value drops only if owned),
// a call result that replaces a live local, early return and `?`.
struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("  drop D({})", self.0);
    }
}

struct P {
    a: D,
    b: D,
}

fn consume(d: D) {
    println!("  consume {}", d.0);
}

fn make(n: i32) -> D {
    println!("  make {n}");
    D(n)
}

fn opt(x: i32) -> Option<i32> {
    if x % 3 == 0 { None } else { Some(x) }
}

fn branches(c: i32) {
    let a = D(10);
    let b = D(11);
    let always = D(12);
    let never = D(13);
    consume(always);
    if c > 0 {
        consume(a);
    } else {
        println!("  keep a");
    }
    match c % 3 {
        0 => consume(b),
        1 => {
            let t = b;
            println!("  t {}", t.0);
        }
        _ => {}
    }
    println!("  end branches {c} {}", never.0);
}

fn loops(n: i32) {
    let mut x = D(20);
    for i in 0..n {
        if i % 2 == 0 {
            consume(x);
            x = make(30 + i);
        }
        let y = D(40 + i);
        if i == 1 {
            consume(y);
            continue;
        }
        if i == 3 {
            break;
        }
        println!("  iter {i}");
    }
    let mut z = D(50);
    let mut k = 0;
    while k < n {
        k += 1;
        z = D(50 + k);
    }
    println!("  end loops {n} {} {}", x.0, z.0);
}

fn partial(c: bool) {
    let mut p = P { a: D(60), b: D(61) };
    if c {
        consume(p.a);
        p.a = D(62);
    }
    let q = P { a: D(63), b: D(64) };
    if c {
        consume(q.b);
    }
    let r = P { a: D(65), b: D(66) };
    let P { a, b: _ } = r;
    println!("  end partial {c} {} {}", p.a.0, a.0);
}

fn reassign(c: i32) {
    let mut x = D(70);
    if c == 1 {
        consume(x);
    }
    x = D(71);
    let mut y: Option<D> = None;
    if c == 2 {
        y = Some(D(72));
    }
    y = Some(make(73));
    let mut w = D(74);
    if c != 0 {
        consume(w);
    }
    w = make(75);
    println!("  end reassign {c} {} {} {}", x.0, y.is_some(), w.0);
}

fn early(c: i32) -> Option<i32> {
    let a = D(80);
    let b = D(81);
    if c == 0 {
        consume(a);
        return None;
    }
    let v = opt(c)?;
    if c == 2 {
        consume(b);
    }
    println!("  end early {c}");
    Some(v)
}

fn main() {
    for c in 0..3 {
        println!("branches {c}");
        branches(c);
    }
    for n in [0, 2, 5] {
        println!("loops {n}");
        loops(n);
    }
    for c in [true, false] {
        println!("partial {c}");
        partial(c);
    }
    for c in 0..3 {
        println!("reassign {c}");
        reassign(c);
    }
    for c in 0..4 {
        println!("early {c}");
        println!("  {:?}", early(c));
    }
}
