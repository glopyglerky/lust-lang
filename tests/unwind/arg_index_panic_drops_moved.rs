// A call argument read through an index panics (bounds check) after an earlier argument
// moved a value: the callee never starts, so the moved value drops once in the caller's
// cleanup, before the panic is caught. Fn calls, method calls, two moved arguments, the
// indexed argument first, a Vec and a slice index, a two-phase `&mut self` receiver whose own
// Vec the index reads, and the same calls when the index is in range (the callee drops the
// value).
use std::hint::black_box;
use std::panic::catch_unwind;

struct Big([u64; 3], &'static str);

impl Drop for Big {
    fn drop(&mut self) {
        println!("drop {}", self.1);
    }
}

#[inline(never)]
fn take(_b: Big, n: u64) -> u64 {
    n
}

#[inline(never)]
fn take2(_b: Big, _c: Big, n: u64) -> u64 {
    n
}

#[inline(never)]
fn first(n: u64, _b: Big) -> u64 {
    n
}

struct S;

struct Bag {
    items: Vec<u64>,
}

impl Bag {
    #[inline(never)]
    fn put(&mut self, _b: Big, n: u64) -> u64 {
        self.items.push(n);
        n + self.items.len() as u64
    }
}

impl S {
    #[inline(never)]
    fn eat(&self, _b: Big, n: u64) -> u64 {
        n + 1
    }
}

fn run(i: usize) {
    let r = catch_unwind(move || {
        let b = Big([1, 2, 3], "fn");
        let a = [7u64];
        take(b, a[i])
    });
    println!("fn caught {}", r.is_err());
    let r = catch_unwind(move || {
        let b = Big([1, 2, 3], "first");
        let c = Big([4, 5, 6], "second");
        let v = vec![7u64];
        take2(b, c, v[i])
    });
    println!("two caught {}", r.is_err());
    let r = catch_unwind(move || {
        let b = Big([1, 2, 3], "after");
        let a = [7u64];
        first(a[i], b)
    });
    println!("first caught {}", r.is_err());
    let r = catch_unwind(move || {
        let b = Big([1, 2, 3], "method");
        let v = vec![7u64];
        let s: &[u64] = &v;
        S.eat(b, s[i])
    });
    println!("method caught {}", r.is_err());
    let r = catch_unwind(move || {
        let b = Big([1, 2, 3], "two-phase");
        let mut bag = Bag { items: vec![7] };
        bag.put(b, bag.items[i])
    });
    println!("two-phase caught {}", r.is_err());
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    run(black_box(1));
    run(black_box(0));
}
