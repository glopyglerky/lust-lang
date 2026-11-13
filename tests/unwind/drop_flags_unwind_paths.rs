// Drop flags on the unwind paths (static drop flags, `ir::dropflags`): a panic drops exactly the
// locals still owned where it starts. A local moved on some paths before the panic; a call
// result that would replace a live local (the old value drops); a callee that owns a moved
// argument when it panics (the caller does not drop it again); one cleanup reached from two
// calls with the local owned before one and moved before the other; a loop that moves and
// reinitializes, with a value made per iteration; partial moves of struct fields.
use std::panic;

struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("  drop D({})", self.0);
    }
}

struct P {
    a: D,
    b: D,
}

fn consume_or_panic(d: D, p: bool) {
    if p {
        panic!("consume {}", d.0);
    }
    println!("  consume {}", d.0);
}

fn make_or_panic(n: i32, p: bool) -> D {
    if p {
        panic!("make {n}");
    }
    D(n)
}

fn maybe_panic(p: bool, what: i32) {
    if p {
        panic!("at {what}");
    }
}

fn moved_then_panic(c: i32) {
    let a = D(1);
    let b = D(2);
    if c % 2 == 0 {
        drop(a);
    }
    maybe_panic(c > 0, 1);
    drop(b);
}

fn call_result_replaces(c: i32) {
    let mut x = D(10);
    if c == 0 {
        drop(x);
        x = D(11);
    }
    x = make_or_panic(12, c > 0);
    println!("  x {}", x.0);
}

fn callee_owns_arg(c: i32) {
    let a = D(20);
    let b = D(21);
    consume_or_panic(a, c == 1);
    if c == 2 {
        consume_or_panic(b, true);
    }
    println!("  after {c}");
}

fn shared_cleanup(c: i32) {
    let a = D(30);
    let b = D(31);
    maybe_panic(c == 0, 2);
    if c != 1 {
        drop(a);
    }
    maybe_panic(c >= 1, 3);
    drop(b);
}

fn loop_panic(n: i32) {
    let mut x = D(40);
    for i in 0..4 {
        let y = D(50 + i);
        if i % 2 == 1 {
            drop(x);
            x = D(41 + i);
        }
        if i == n {
            panic!("loop {i}");
        }
        if i == 2 {
            drop(y);
        }
    }
    println!("  loop done {}", x.0);
}

fn partial_panic(c: i32) {
    let mut p = P { a: D(60), b: D(61) };
    if c > 0 {
        drop(p.a);
    }
    if c == 2 {
        p.a = D(62);
    }
    maybe_panic(true, 4);
    println!("  {}", p.b.0);
}

fn main() {
    panic::set_hook(Box::new(|info| {
        let m = info.payload().downcast_ref::<String>().map(|s| s.as_str()).unwrap_or("?");
        println!("  panic: {m}");
    }));
    let cases: [(&str, fn(i32)); 6] = [
        ("moved_then_panic", moved_then_panic),
        ("call_result_replaces", call_result_replaces),
        ("callee_owns_arg", callee_owns_arg),
        ("shared_cleanup", shared_cleanup),
        ("loop_panic", loop_panic),
        ("partial_panic", partial_panic),
    ];
    for (name, f) in cases {
        for c in 0..4 {
            println!("{name} {c}");
            let r = panic::catch_unwind(|| f(c));
            println!("  caught {}", r.is_err());
        }
    }
}
