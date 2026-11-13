// For-loop item drops on unwind exits (SOL-COVERAGE-4 D2): a panic in a
// call-free body (an overflow), in a calling body, in `next`, and in an
// adapter closure. The current item, the bound rest, and the iterator drop
// during unwinding, in the order std gives. `next` and every Drop print.
use std::panic;

struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("  drop D({})", self.0);
    }
}
struct It(i32, i32);
impl Iterator for It {
    type Item = (D, D);
    fn next(&mut self) -> Option<(D, D)> {
        if self.0 == 0 {
            return None;
        }
        self.0 -= 1;
        if self.0 == self.1 {
            panic!("next {}", self.0);
        }
        println!("  next {}", self.0);
        Some((D(self.0), D(self.0 + 10)))
    }
}
impl Drop for It {
    fn drop(&mut self) {
        println!("  drop It({})", self.0);
    }
}

#[inline(never)]
fn boom(n: i32) {
    if n == 2 {
        panic!("boom {n}");
    }
}

fn main() {
    panic::set_hook(Box::new(|_| println!("  panicked")));
    println!("overflow in a call-free body");
    let r = panic::catch_unwind(|| {
        let mut k: u8 = 250;
        for (a, _) in It(4, 99) {
            k += a.0 as u8 + 1;
        }
        k
    });
    println!("  ok {}", r.is_ok());
    println!("panic in a calling body");
    let r = panic::catch_unwind(|| {
        for (a, b) in It(4, 99) {
            boom(a.0);
            println!("  body {} {}", a.0, b.0);
        }
    });
    println!("  ok {}", r.is_ok());
    println!("panic in next");
    let r = panic::catch_unwind(|| {
        let mut k = 0;
        for _ in It(4, 1) {
            k += 1;
        }
        k
    });
    println!("  ok {}", r.is_ok());
    println!("panic in an adapter closure");
    let r = panic::catch_unwind(|| {
        let mut k = 0;
        for x in It(4, 99).map(|(a, b)| {
            if a.0 == 1 {
                panic!("map {}", b.0);
            }
            a
        }) {
            k += x.0;
        }
        k
    });
    println!("  ok {}", r.is_ok());
}
