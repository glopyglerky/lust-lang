// Under a `LazyCell<T>` or `LazyCell<T, fn() -> T>` annotation the
// init argument coerces to the fn pointer type: the expected return
// type binds `F` before the closure or fn item argument checks.
use std::cell::LazyCell;
use std::sync::LazyLock;

fn seed() -> u32 {
    41
}

struct Reg {
    lazy: LazyCell<u32, fn() -> u32>,
}

struct W<F> {
    f: Option<F>,
}

impl<F> W<F> {
    fn new(f: F) -> Self {
        W { f: Some(f) }
    }
}

fn main() {
    let a: LazyCell<u32> = LazyCell::new(|| 5);
    let b: LazyCell<u32, fn() -> u32> = LazyCell::new(seed);
    let c: LazyLock<u32> = LazyLock::new(|| 7);
    let d: LazyLock<u32, fn() -> u32> = LazyLock::new(seed);
    let r = Reg { lazy: LazyCell::new(seed) };
    let w: W<fn() -> u32> = W::new(|| 5);
    let v: W<fn() -> u32> = W::new(seed);
    println!("{} {} {} {} {} {} {}", *a, *b, *c, *d, *r.lazy, (w.f.unwrap())(), (v.f.unwrap())());
}
