// `mem::forget` takes ownership and never runs the value's drop; the
// other `std::mem` free fns by every path form (W-STD-C).
use std::mem;
use std::mem::{align_of_val, forget};

struct Loud(&'static str);

impl Drop for Loud {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn main() {
    let a = Loud("a");
    let b = Loud("b");
    let c = Loud("c");
    mem::forget(a);
    forget(b);
    // Only values with no heap buffer: a forgotten buffer is a real
    // leak, which the SANITIZE=1 difftest run reports.
    core::mem::forget(Loud("core path"));
    std::mem::forget([Loud("x"), Loud("y")]);
    let mut d = Loud("d");
    let old = mem::replace(&mut d, Loud("e"));
    forget(old);
    println!("{} {}", align_of_val(&5u64), mem::align_of_val(&[1u16, 2]));
    let mut x = 3;
    let mut y = 4;
    mem::swap(&mut x, &mut y);
    let t = mem::take(&mut x);
    println!("{} {} {}", x, y, t);
    drop(c);
    println!("end");
}
