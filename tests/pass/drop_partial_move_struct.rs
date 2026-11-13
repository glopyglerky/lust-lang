// Partial moves out of structs and tuples whose fields have a user
// `Drop` impl: each value drops exactly once, the remaining fields
// drop in declaration order at scope end, and a moved field that is
// assigned again drops its new value (review group 11).
struct D(u8);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

struct P {
    a: D,
    b: D,
    c: D,
}

struct Q {
    a: D,
    b: D,
}

struct N {
    q: Q,
    c: D,
    t: (D, D),
}

fn eat(d: D) {
    println!("eat {}", d.0);
}

fn conditional(c: bool) {
    let p = P { a: D(1), b: D(2), c: D(3) };
    if c {
        let x = p.b;
        println!("took {}", x.0);
    }
    println!("end {}", c);
}

fn refill() {
    let mut q = Q { a: D(4), b: D(5) };
    let x = q.a;
    println!("moved {}", x.0);
    q.a = D(6);
    q.a = D(7);
    println!("refilled {}", q.a.0);
}

fn whole_reassign() {
    let mut q = Q { a: D(8), b: D(9) };
    let x = q.a;
    println!("moved {}", x.0);
    q = Q { a: D(10), b: D(11) };
    println!("reassigned {} {}", q.a.0, q.b.0);
}

fn nested() {
    let n = N { q: Q { a: D(12), b: D(13) }, c: D(14), t: (D(15), D(16)) };
    let x = n.q.b;
    let y = n.t.0;
    println!("moved {} {}", x.0, y.0);
}

fn param(p: P) {
    eat(p.b);
    println!("param end");
}

fn in_loop() {
    let mut q = Q { a: D(20), b: D(21) };
    let mut i = 0;
    while i < 3 {
        let x = q.a;
        println!("iter {} took {}", i, x.0);
        q.a = D(30 + i);
        i += 1;
    }
    println!("loop end");
}

fn order() {
    let p = P { a: D(40), b: D(41), c: D(42) };
    let y = p.c;
    let x = p.a;
    println!("{} {}", x.0, y.0);
}

fn update() {
    let p = P { a: D(50), b: D(51), c: D(52) };
    let q = P { a: D(53), ..p };
    println!("{} {} {}", q.a.0, q.b.0, q.c.0);
}

fn main() {
    conditional(true);
    conditional(false);
    refill();
    whole_reassign();
    nested();
    param(P { a: D(17), b: D(18), c: D(19) });
    in_loop();
    order();
    update();
}
