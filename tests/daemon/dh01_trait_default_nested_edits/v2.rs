fn k() -> i64 { 1 }

fn f0() -> i64 {
    fn y() -> i64 { 3 }
    fn z() -> i64 { 7 + y() }
    9 + z()
}

trait Tr {
    fn t(&self) -> i64;
    fn d(&self) -> i64 {
        fn h(x: &i64) -> bool { *x > 1 }
        fn m(x: i64) -> i64 { x * 10 }
        fn g() -> i64 { k() }
        self.t() + [1i64, 2, 3].iter().copied().filter(h).map(m).sum::<i64>() + g()
    }
}

impl Tr for i64 {
    fn t(&self) -> i64 {
        *self
    }
}

fn main() {
    println!("{}", f0() + 5i64.d());
}
