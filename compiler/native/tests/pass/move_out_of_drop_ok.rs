// Moves that a `Drop` impl allows: the whole value, a Copy field,
// borrows of fields, and a field of a field-less-Drop parent.
struct D(u8);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

struct S {
    a: D,
    n: u8,
}

impl Drop for S {
    fn drop(&mut self) {
        println!("drop S {}", self.a.0);
    }
}

struct P {
    d: D,
    e: D,
}

fn take(s: S) -> u8 {
    s.n
}

fn main() {
    let s = S { a: D(1), n: 2 };
    let n = s.n;
    let r = &s.a;
    println!("{} {}", n, r.0);
    let t = s;
    println!("{}", take(t));
    let e = S { a: D(3), n: 4 };
    match &e {
        S { a, n } => println!("{} {}", a.0, n),
    }
    let p = P { d: D(5), e: D(6) };
    let x = p.d;
    println!("{}", x.0);
}
