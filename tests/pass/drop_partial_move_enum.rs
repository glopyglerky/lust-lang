// Moves out of enum payloads and `Option`s whose contents have a
// user `Drop` impl: the moved part never drops again, the rest of
// that variant drops at scope end, and other variants drop whole.
struct D(u8);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

enum E {
    A(D, D),
    B(D),
    C,
}

struct Q {
    x: D,
    y: Option<D>,
}

fn by_variant(e: E) {
    match e {
        E::A(x, _) => println!("A {}", x.0),
        E::B(_) => println!("B"),
        E::C => println!("C"),
    }
    println!("after match");
}

fn options() {
    let o = Some(D(5));
    if let Some(x) = o {
        println!("x {}", x.0);
    }
    let t = Some((D(6), D(7)));
    if let Some((a, _)) = t {
        println!("a {}", a.0);
    }
    let n: Option<(D, D)> = None;
    if let Some((a, _)) = n {
        println!("a {}", a.0);
    }
    println!("options end");
}

fn field_option() {
    let q = Q { x: D(8), y: Some(D(9)) };
    match q.y {
        Some(d) => println!("y {}", d.0),
        None => println!("none"),
    }
    println!("after");
}

fn main() {
    by_variant(E::A(D(1), D(2)));
    by_variant(E::B(D(3)));
    by_variant(E::C);
    options();
    field_option();
}
