// Fns nested in trait default method bodies, next to nested fns of the
// same names in free fns, impl methods, closures, and generic fns. The
// token scan once skipped trait bodies, so these fns had no slot and the
// driver stopped with an internal DefPathHash collision (exit 3).
fn f0() -> i64 {
    fn z() -> i64 { 7 }
    9 + z()
}

trait Tr {
    fn t(&self) -> i64;
    fn d(&self) -> i64 {
        fn h(x: &i64) -> bool { *x > 1 }
        fn m(x: i64) -> i64 { x * 10 }
        self.t() + [1i64, 2, 3].iter().copied().filter(h).map(m).sum::<i64>()
    }
    fn e(&self) -> i64 {
        fn h() -> i64 {
            fn h() -> i64 { 5 }
            h() + 1
        }
        struct P(i64);
        let c = |x: i64| {
            fn m(y: i64) -> i64 { y + 100 }
            m(x)
        };
        h() + P(2).0 + c(3)
    }
}

trait Tr2 {
    fn d(&self) -> i64 {
        fn h(x: &i64) -> bool { *x > 2 }
        [1i64, 2, 3].iter().copied().filter(h).count() as i64
    }
}

impl Tr for i64 {
    fn t(&self) -> i64 {
        fn h() -> i64 { 1000 }
        *self + h()
    }
}
impl Tr2 for i64 {}

struct S;
impl S {
    fn d(&self) -> i64 {
        fn h() -> i64 { 8 }
        h()
    }
}

fn g<T: Copy>(t: T) -> T {
    fn m() -> i64 { 9 }
    let _ = m();
    t
}

fn main() {
    fn m() -> i64 { 4 }
    println!("{}", f0() + Tr::d(&5i64) + f2());
    println!("{} {} {} {} {}", Tr::e(&0i64), Tr2::d(&0i64), S.d(), g(6), m());
}

fn f2() -> i64 {
    5
}
