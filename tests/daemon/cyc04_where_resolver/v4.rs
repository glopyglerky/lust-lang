const ROUTE: u8 = 0;

struct Acc {
    v: i64,
}

impl Acc {
    fn keep<T>(&self, t: T) -> (T, T)
    where
        T: Clone,
    {
        (t.clone(), t)
    }
}

fn pick<T>(t: T) -> T
where
    T: Clone,
{
    t.clone()
}

fn show(xs: &[i64]) {
    println!("{}", xs.len());
}

fn main() {
    let _r = ROUTE;
    let a = Acc { v: 2 };
    let s = pick(String::from("hi"));
    let (x, y) = a.keep(vec![a.v]);
    show(&x);
    println!("{} {}", s, y.len());
}
