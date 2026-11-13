fn inc(x: i64) -> i64 {
    x + 1
}
fn dbl(x: i64) -> i64 {
    x * 2
}
fn apply(f: fn(i64) -> i64, x: i64) -> i64 {
    f(x)
}
fn choose(c: bool) -> fn(i64) -> i64 {
    if c { inc } else { dbl }
}
struct Holder {
    f: fn(i64) -> i64,
}
struct Ctr {
    n: i64,
}
impl Ctr {
    fn get(&self) -> i64 {
        self.n
    }
    fn make(n: i64) -> Ctr {
        Ctr { n }
    }
}
fn outer() -> fn(i64) -> i64 {
    fn inner(x: i64) -> i64 {
        x + 100
    }
    inner
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
    let p: fn(i64) -> i64 = inc;
    println!("{} {} {}", p(1), apply(dbl, 3), apply(p, 4));
    println!("{} {}", choose(true)(10), choose(false)(10));
    let s = Holder { f: dbl };
    let t = (inc as fn(i64) -> i64, 7);
    println!("{} {}", (s.f)(5), (t.0)(t.1));
    let mut q: fn(i64) -> i64 = inc;
    q = dbl;
    let h = g;
    println!("{} {}", q(2), h(2));
    println!("{}", (inc as fn(i64) -> i64) == p);
    let arr: [fn(i64) -> i64; 2] = [inc, dbl];
    let r = &arr[1];
    println!("{} {}", r(6), arr[0](6));
    let get = Ctr::get;
    let ctr = Ctr::make(41);
    println!("{}", get(&ctr) + 1);
    println!("{}", outer()(1));
    let o: Option<fn(i64) -> i64> = Some(inc);
    println!("{}", o.unwrap()(4));
    let pair: (fn(i64) -> i64, i64) = (dbl, 3);
    println!("{}", (pair.0)(pair.1));
}
