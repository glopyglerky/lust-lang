// Destructuring assignment evaluation order, byte for byte with
// rustc's desugaring: the right-hand side evaluates once, written
// elements move into binding temporaries in target order, what
// remains of a temporary scrutinee drops, then each target place
// evaluates just before its own write. A scrutinee that is a place
// rooted in a local binds out of that place: discarded elements
// stay in it and drop with its owner.
struct G(i32);

impl Drop for G {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

struct Wrap {
    inner: G,
}

struct W {
    pair: (G, G),
    other: G,
}

fn f() -> usize {
    println!("idx f");
    0
}

fn g() -> usize {
    println!("idx g");
    1
}

fn make(n: i32) -> G {
    println!("make {}", n);
    G(n)
}

fn tup() -> (G, G) {
    println!("call tup");
    (make(20), make(21))
}

fn wrap() -> W {
    println!("call wrap");
    W { pair: (make(20), make(21)), other: make(22) }
}

fn slot(a: &mut [G; 2]) -> &mut G {
    println!("call slot");
    &mut a[1]
}

fn get(w: &mut Wrap) -> &mut Wrap {
    println!("call get");
    w
}

fn value_rhs() {
    println!("-- value");
    let mut a = [G(10), G(11)];
    (a[f()], _) = (make(20), make(21));
    println!("done {}", a[0].0);
    (_, a[f()]) = (make(30), make(31));
    println!("done {}", a[0].0);
}

fn two_places() {
    println!("-- two places");
    let mut a = [G(10), G(11)];
    (a[f()], a[g()]) = (make(20), make(21));
    println!("done {} {}", a[0].0, a[1].0);
    // The same place twice: the second write overwrites the first.
    (a[f()], a[f()]) = (make(30), make(31));
    println!("done {}", a[0].0);
}

fn old_drops_interleave() {
    println!("-- interleave");
    let mut x = G(9);
    let mut a = [G(10), G(11)];
    (x, a[f()]) = (make(20), make(21));
    println!("done {} {}", x.0, a[0].0);
}

fn nested() {
    println!("-- nested");
    let mut a = [G(10), G(11)];
    ((a[f()], _), a[g()]) = ((make(20), make(21)), make(22));
    println!("done {} {}", a[0].0, a[1].0);
    (a[f()], _) = (make(30), (make(31), make(32)));
    println!("done {}", a[0].0);
}

fn call_rhs() {
    println!("-- call rhs");
    let mut a = [G(10), G(11)];
    (a[f()], _) = tup();
    println!("done {}", a[0].0);
}

fn place_rhs() {
    println!("-- place rhs");
    let mut a = [G(10), G(11)];
    let t = (make(20), make(21));
    println!("t made");
    (a[f()], _) = t;
    println!("reuse {}", t.1.0);
    println!("done {}", a[0].0);
}

fn place_rhs_xy() {
    println!("-- xy rhs");
    let mut a = [G(10), G(11)];
    let x = make(20);
    let y = make(21);
    println!("xy made");
    (a[f()], _) = (x, y);
    println!("done {}", a[0].0);
}

fn field_rhs() {
    println!("-- field rhs");
    let mut a = [G(10), G(11)];
    let w = W { pair: (make(20), make(21)), other: make(22) };
    println!("w made");
    (a[f()], _) = w.pair;
    println!("reuse {} {}", w.pair.1.0, w.other.0);
    println!("done {}", a[0].0);
}

fn field_of_temp() {
    println!("-- field of temp");
    let mut a = [G(10), G(11)];
    (a[f()], _) = wrap().pair;
    println!("done {}", a[0].0);
}

fn overlap() {
    println!("-- overlap");
    let mut t = (make(20), make(21));
    (t.1, t.0) = t;
    println!("done {} {}", t.0.0, t.1.0);
    let mut a = [G(10), G(11)];
    (a[f()], _) = t;
    println!("mid {} {}", a[0].0, t.1.0);
    t.0 = make(22);
    println!("done {} {}", t.0.0, t.1.0);
}

fn compound_places() {
    println!("-- compound places");
    let mut a = [G(10), G(11)];
    (*slot(&mut a), _) = (make(20), make(21));
    println!("done {}", a[1].0);
    let mut w = Wrap { inner: G(10) };
    ((*get(&mut w)).inner, _) = (make(30), make(31));
    println!("done {}", w.inner.0);
}

fn complex_index() {
    println!("-- complex index");
    let mut a = [G(10), G(11)];
    let mut v = vec![G(30)];
    (a[{
        println!("idx block");
        v.push(make(99));
        g()
    }], _) = (make(20), make(21));
    println!("done {} len{}", a[1].0, v.len());
}

fn main() {
    value_rhs();
    two_places();
    old_drops_interleave();
    nested();
    call_rhs();
    place_rhs();
    place_rhs_xy();
    field_rhs();
    field_of_temp();
    overlap();
    compound_places();
    complex_index();
}
