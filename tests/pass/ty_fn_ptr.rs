fn apply(f: fn(i64) -> i64, x: i64) -> i64 {
    f(x)
}

fn inc(x: i64) -> i64 {
    x + 1
}

fn main() {
    println!("{}", apply(inc, 2));
}
