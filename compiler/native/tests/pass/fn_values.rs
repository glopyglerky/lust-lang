fn inc(x: i64) -> i64 {
    x + 1
}

fn dbl(x: i64) -> i64 {
    x * 2
}

#[derive(Debug)]
enum E {
    A(u8, u16),
    B,
}

#[derive(Debug)]
struct P(i32, i32);

fn apply(f: fn(i64) -> i64, v: i64) -> i64 {
    f(v)
}

fn main() {
    let g = dbl;
    println!("{} {}", g(2), inc(1));
    let fs = [inc, dbl];
    let mut i = 0;
    while i < 2 {
        println!("{}", fs[i](5));
        i += 1;
    }
    let c = true;
    let f = if c { inc } else { dbl };
    println!("{}", f(10));
    let f2 = inc;
    println!("{}", std::mem::size_of_val(&f2));
    let f3: fn(i64) -> i64 = inc;
    println!("{}", std::mem::size_of_val(&f3));
    println!("{}", apply(inc, 4));
    let mk = E::A;
    println!("{:?}", mk(1, 2));
    let mkp = P;
    println!("{:?}", mkp(3, 4));
}
