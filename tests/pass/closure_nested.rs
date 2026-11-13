// Nested closures: the outer closure captures what the inner one needs.
fn main() {
    let base = 10i64;
    let mut total = 0i64;
    let mut outer = |n: i64| {
        let scale = 2i64;
        let inner = |m: i64| m * scale + base;
        total += inner(n);
        inner(1)
    };
    println!("{}", outer(3));
    println!("{}", outer(4));
    println!("total {}", total);
    let add = |x: i64| x + base;
    let twice = |x: i64| add(add(x));
    println!("{}", twice(1));
}
