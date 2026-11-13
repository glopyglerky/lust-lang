const ROUTE: u8 = 0;

#[derive(Clone, Debug, PartialEq)]
struct P {
    x: i64,
}

fn same<T: PartialEq>(a: &T, b: &T) -> bool {
    a == b
}

fn show(xs: &[i64]) {
    println!("{:?}", xs);
}

fn main() {
    let _r = ROUTE;
    let p = P { x: 5 };
    let q = p;
    let pair = (p, Some(q));
    show(&[p.x, q.x]);
    println!("{:?} {}", pair, same(&p, &q));
}
