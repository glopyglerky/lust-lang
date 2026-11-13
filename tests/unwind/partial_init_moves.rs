// A panic part way through building or moving values: the parts already built drop in
// reverse order, moved-out parts do not drop, and conditional moves follow the drop flags.
use std::panic::catch_unwind;
struct D(u32);
impl Drop for D {
    fn drop(&mut self) { println!("drop {}", self.0); }
}
struct S { a: D, b: D, c: D }
fn boom(n: u32) -> D {
    if n > 0 { panic!("boom {n}"); }
    D(0)
}
fn take(d: D) { println!("take {}", d.0); }
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let r = catch_unwind(|| { let _s = S { a: D(1), b: D(2), c: boom(1) }; });
    println!("struct {}", r.is_err());
    let r = catch_unwind(|| { let _t = (D(3), D(4), boom(2), D(5)); });
    println!("tuple {}", r.is_err());
    let r = catch_unwind(|| { let _a = [D(6), D(7), boom(3)]; });
    println!("array {}", r.is_err());
    let r = catch_unwind(|| { let _v = vec![D(8), D(9), boom(4)]; });
    println!("vec {}", r.is_err());
    let r = catch_unwind(|| { let _o = Some((D(10), boom(5))); });
    println!("enum {}", r.is_err());
    let r = catch_unwind(|| {
        let s = S { a: D(11), b: D(12), c: D(13) };
        let x = s.a;
        take(s.c);
        let _ = boom(6);
        drop(x);
    });
    println!("partial move {}", r.is_err());
    for flag in [false, true] {
        let r = catch_unwind(move || {
            let s = S { a: D(14), b: D(15), c: D(16) };
            if flag { take(s.b); }
            let _ = boom(7);
            drop(s.a);
        });
        println!("conditional {flag} {}", r.is_err());
    }
    let r = catch_unwind(|| {
        let mut v = vec![D(17), D(18), D(19)];
        let mut held = Vec::new();
        for i in 0..3 {
            if i == 2 { let _ = boom(8); }
            held.push(v.remove(0));
        }
        drop(held);
    });
    println!("loop moves {}", r.is_err());
    let r = catch_unwind(|| {
        let a = D(20);
        let b;
        let _ = boom(9);
        b = D(21);
        drop((a, b));
    });
    println!("late init {}", r.is_err());
    let r = catch_unwind(|| {
        let mut x = D(22);
        x = { let _ = boom(10); D(23) };
        drop(x);
    });
    println!("assign rhs {}", r.is_err());
}
