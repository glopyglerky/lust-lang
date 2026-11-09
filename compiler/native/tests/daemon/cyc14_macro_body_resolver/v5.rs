const ROUTE: u8 = 0;

use std::fmt::Write;

fn pick(x: i64) -> bool {
    matches!(x, 1 | 2)
}

fn mk(n: usize) -> Vec<i64> {
    #[allow(unused_variables)]
    let unused = 1;
    vec![0; n]
}

fn show(xs: &[i64]) {
    println!("{:?}", xs);
}

fn label(v: &[i64]) -> String {
    let mut s = String::new();
    write!(s, "{}", v.len()).unwrap();
    s
}

fn main() {
    let _r = ROUTE;
    let v = mk(2);
    show(&v);
    println!("{} {}", pick(v.len() as i64), label(&v));
}
