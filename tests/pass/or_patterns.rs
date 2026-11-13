enum E { A(i32), B(i32), C }
enum W { X(String, i32), Y(String) }

struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) { println!("drop {}", self.0); }
}
enum P { M(D, D), N(D) }

fn at(i: i32) -> Option<E> {
    match i {
        0 => Some(E::A(1)),
        1 => Some(E::B(2)),
        _ => Some(E::C),
    }
}

fn pick(e: E) -> i32 {
    match e {
        E::A(x) | E::B(x) => x + 1,
        E::C => 0,
    }
}

fn main() {
    // Basic alternatives, also through a guard that reads the binding.
    println!("{} {} {}", pick(E::A(1)), pick(E::B(2)), pick(E::C));
    match E::B(9) {
        E::A(x) | E::B(x) if x > 5 => println!("big {}", x),
        E::A(x) | E::B(x) => println!("small {}", x),
        E::C => println!("c"),
    }
    // Nested or-patterns inside tuples, structs, and other or-patterns.
    let t = (E::B(3), 7);
    match t {
        (E::A(x) | E::B(x), y) => println!("t {} {}", x, y),
        (E::C, y) => println!("tc {}", y),
        _ => println!("t other"),
    }
    match (1, 2) {
        ((x, _) | (_, x)) => println!("first {}", x),
    }
    match ((1, 2), 3) {
        (((x @ 1..=3 | x @ 5..=9), _), _) => println!("range {}", x),
        _ => println!("no range"),
    }
    // Guard backtracking: the second alternative binds x = 2.
    match (1, 2) {
        ((x, _) | (_, x)) if x == 2 => println!("two via second"),
        _ => println!("no backtrack"),
    }
    // let, let-else, if let, while let, for.
    let ((x, _) | (_, x)) = (10, 20);
    println!("let {}", x);
    let (E::A(v) | E::B(v)) = E::A(4) else { println!("else"); return };
    println!("lete {}", v);
    if let E::A(x) | E::B(x) = E::B(6) { println!("iflet {}", x); }
    let mut i = 0;
    while let Some(E::A(x) | E::B(x)) = at(i) {
        println!("while {}", x);
        i += 1;
    }
    for (a, _) | (_, a) in [(1, 2), (3, 4)] { println!("for {}", a); }
    // Option and Result nesting.
    match Some(E::B(8)) {
        Some(E::A(x) | E::B(x)) => println!("some {}", x),
        _ => println!("none"),
    }
    let r: Result<i32, i32> = Err(12);
    match r {
        Ok(x) | Err(x) => println!("res {}", x),
    }
    // String payloads: move out of one alternative, keep the other fields.
    match W::X("s".to_string(), 5) {
        W::X(s, _) | W::Y(s) if s.len() == 1 => println!("w {}", s.len()),
        _ => println!("w other"),
    }
    // Guard failure leaves the scrutinee whole for the next arm.
    let e = E::A(2);
    match e {
        E::A(x) | E::B(x) if x > 5 => println!("gf big"),
        E::A(x) => println!("gf a {}", x),
        _ => println!("gf other"),
    }
    // Borrows: shared and mutable bindings through alternatives.
    let e2 = E::B(30);
    match &e2 {
        E::A(x) | E::B(x) => println!("ref {}", x),
        E::C => println!("ref c"),
    }
    let mut e3 = E::A(1);
    match &mut e3 {
        E::A(x) | E::B(x) => *x += 40,
        E::C => {}
    }
    match e3 {
        E::A(x) | E::B(x) => println!("mut {}", x),
        E::C => println!("mut c"),
    }
    // Slice alternatives bind from different positions.
    let v = [1, 2];
    match &v[..] {
        [x] | [x, _] => println!("slice {}", x),
        _ => println!("slice other"),
    }
    // Drop order: the bound field drops at arm end, the unbound one with the scrutinee temp.
    match P::M(D("m1"), D("m2")) {
        P::M(d, _) | P::N(d) => println!("got {}", d.0),
    }
    println!("end");
}
