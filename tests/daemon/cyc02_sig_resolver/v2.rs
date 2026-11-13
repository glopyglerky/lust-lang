const ROUTE: u8 = 0;

fn scale(x: i32) -> i64 {
    x as i64 * 3
}

fn twice<T: Copy>(t: T) -> (T, T) {
    (t, t)
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
