const ROUTE: u8 = 0;

fn make() -> impl std::fmt::Debug {
    vec![1i64, 2]
}

fn wrap() -> impl std::fmt::Debug {
    (make(), 3u8)
}

fn show(xs: &[i64]) {
    println!("{}", xs.len());
}

fn main() {
    let _r = ROUTE;
    let m = make();
    let n = m.clone();
    let w = wrap();
    show(&[1, 2]);
    println!("{:?} {:?} {:?}", m, n, w);
}
