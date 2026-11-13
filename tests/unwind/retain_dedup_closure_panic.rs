// A panic in the closure of `retain`, `retain_mut`, `dedup_by`, or
// `dedup_by_key` keeps every element not yet removed, in order (std's
// `BackshiftOnDrop` and `FillGapOnDrop` guards); nothing drops twice.
use std::panic::{AssertUnwindSafe, catch_unwind};

#[derive(Debug, PartialEq)]
struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("  drop {}", self.0);
    }
}

fn mk() -> Vec<D> {
    vec![D(0), D(1), D(1), D(2), D(3)]
}

fn main() {
    for stop in 1..5 {
        let mut v = mk();
        let mut i = 0;
        let r = catch_unwind(AssertUnwindSafe(|| {
            v.retain(|_| {
                i += 1;
                if i == stop {
                    panic!("stop");
                }
                i % 2 == 0
            })
        }));
        println!("retain {stop}: err={} v={:?}", r.is_err(), v);
        let mut v = mk();
        let mut i = 0;
        let r = catch_unwind(AssertUnwindSafe(|| {
            v.retain_mut(|d| {
                i += 1;
                d.0 += 10;
                if i == stop {
                    panic!("stop");
                }
                i % 2 == 1
            })
        }));
        println!("retain_mut {stop}: err={} v={:?}", r.is_err(), v);
        let mut v = mk();
        let mut i = 0;
        let r = catch_unwind(AssertUnwindSafe(|| {
            v.dedup_by(|a, b| {
                i += 1;
                if i == stop {
                    panic!("stop");
                }
                a == b
            })
        }));
        println!("dedup_by {stop}: err={} v={:?}", r.is_err(), v);
        let mut v = mk();
        let mut i = 0;
        let r = catch_unwind(AssertUnwindSafe(|| {
            v.dedup_by_key(|a| {
                i += 1;
                if i == stop {
                    panic!("stop");
                }
                format!("{}", a.0)
            })
        }));
        println!("dedup_by_key {stop}: err={} v={:?}", r.is_err(), v);
    }
}
