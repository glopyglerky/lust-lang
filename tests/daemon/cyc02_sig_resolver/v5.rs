const ROUTE: u8 = 0;

fn scale(x: i64) -> i64 {
    x * 3
}

fn twice<T: Clone>(t: T) -> (T, T) {
    (t.clone(), t)
}

fn show(xs: &[i64]) {
    println!("{:?}", xs);
}

fn main() {
    let _r = ROUTE;
    let n: i64 = 4;
    let (a, b) = twice(vec![1i64, 2]);
    show(&a);
    println!("{} {}", scale(n), b.len());
}
