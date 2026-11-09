// Destructuring assignment with `..` rest in tuple, array, and
// tuple-struct targets, full discards, unit targets, and targets
// inside closures. Rest elements of a value drop in order after
// the binding moves and before the first target place evaluates;
// rest elements of an owned place stay in it.
struct G(i32);

impl Drop for G {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

struct U;

struct PairB(i32, i32);

struct SB {
    x: i32,
    y: i32,
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

fn u() {
    println!("call u");
}

fn make_u() -> U {
    println!("call make_u");
    U
}

fn array_rest() {
    println!("-- array rest");
    let mut a = [G(10), G(11)];
    [a[f()], .., a[g()]] = [make(20), make(21), make(22), make(23)];
    println!("done {} {}", a[0].0, a[1].0);
    [.., a[f()]] = [make(30), make(31), make(32)];
    println!("done {}", a[0].0);
    [a[f()], ..] = [make(40), make(41), make(42)];
    println!("done {}", a[0].0);
}

fn tuple_rest() {
    println!("-- tuple rest");
    let mut a = [G(10), G(11)];
    (a[f()], .., a[g()]) = (make(20), make(21), make(22), make(23));
    println!("done {} {}", a[0].0, a[1].0);
    (a[f()], a[g()], ..) = (make(30), make(31));
    println!("done {} {}", a[0].0, a[1].0);
}

fn place_rhs_rest() {
    println!("-- place rhs rest");
    let mut a = [G(10), G(11)];
    let src = [make(20), make(21), make(22)];
    println!("src made");
    [a[f()], ..] = src;
    println!("done {}", a[0].0);
}

fn all_discard() {
    println!("-- all discard");
    (_, _) = (make(20), make(21));
    println!("mid");
    [_, .., _] = [make(30), make(31), make(32), make(33)];
    println!("done");
}

fn unit_targets() {
    println!("-- unit targets");
    () = u();
    U = make_u();
    let mut a = 0;
    (a, (), _) = (7, u(), make(20));
    println!("done {}", a);
}

fn infer_from_place() {
    println!("-- infer");
    let mut a: i64 = 0;
    PairB2(a, _) = PairB2(40, 2);
    let mut arr: [i64; 2] = [0, 0];
    [arr[0], ..] = [7, 8, 9];
    println!("done {} {}", a, arr[0]);
}

struct PairB2<T>(T, T);

fn vec_index_target() {
    println!("-- vec index target");
    let mut v = vec![G(10), G(11)];
    [v[f()], ..] = [make(20), make(21)];
    println!("done {}", v[0].0);
}

fn closures() {
    println!("-- closures");
    let mut a = 0;
    let mut b = 0;
    let mut c = || {
        (a, _) = (3, 4);
        PairB(b, _) = PairB(5, 6);
    };
    c();
    let mut d = || {
        SB { x: a, .. } = SB { x: 7, y: 8 };
        [b, ..] = [9, 10, 11];
    };
    d();
    println!("done {} {}", a, b);
}

fn main() {
    array_rest();
    tuple_rest();
    place_rhs_rest();
    all_discard();
    unit_targets();
    infer_from_place();
    vec_index_target();
    closures();
}
