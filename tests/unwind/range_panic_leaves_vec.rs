// A range or index panic in a Vec, String, or slice method leaves the
// value as it was: std checks the bounds before it changes anything
// (splice, drain, copy_within, insert). A value the method owns, or a
// value moved into an indexed place, drops as the panic unwinds.
use std::panic::{AssertUnwindSafe, catch_unwind};

#[derive(Debug)]
struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("  drop {}", self.0);
    }
}

fn main() {
    let k = std::env::args().count() - 1;
    let mut v = vec![1, 2, 3];
    let r = catch_unwind(AssertUnwindSafe(|| {
        v.splice((2 + k)..(9 + k), [7]);
    }));
    println!("splice err={} v={:?} cap={}", r.is_err(), v, v.capacity());
    let r = catch_unwind(AssertUnwindSafe(|| {
        v.splice((9 + k)..(10 + k), [7]);
    }));
    println!("splice past end err={} v={:?}", r.is_err(), v);
    let r = catch_unwind(AssertUnwindSafe(|| {
        v.drain((1 + k)..(9 + k));
    }));
    println!("drain err={} v={:?}", r.is_err(), v);
    let mut w = [1, 2, 3, 4];
    let r = catch_unwind(AssertUnwindSafe(|| {
        w.copy_within((1 + k)..(9 + k), 0);
    }));
    println!("copy_within err={} w={:?}", r.is_err(), w);
    let mut s = String::from("h\u{e9}llo");
    let r = catch_unwind(AssertUnwindSafe(|| {
        s.drain((1 + k)..(9 + k));
    }));
    println!("str drain err={} s={:?}", r.is_err(), s);
    let r = catch_unwind(AssertUnwindSafe(|| {
        s[(2 + k)..=(6 + k)].make_ascii_uppercase();
    }));
    println!("str slice err={} s={:?}", r.is_err(), s);
    let mut d = vec![D(1), D(2), D(3)];
    let r = catch_unwind(AssertUnwindSafe(|| {
        d.splice((2 + k)..(9 + k), [D(7)]);
    }));
    println!("splice drop err={} len={}", r.is_err(), d.len());
    let r = catch_unwind(AssertUnwindSafe(|| {
        d.insert(5 + k, D(8));
    }));
    println!("insert err={} len={}", r.is_err(), d.len());
    let r = catch_unwind(AssertUnwindSafe(|| {
        d[5 + k] = D(9);
    }));
    println!("index set err={} d={:?}", r.is_err(), d);
}
