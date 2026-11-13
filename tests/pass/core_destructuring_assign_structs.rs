// Destructuring assignment with tuple structs, braced structs,
// struct `..` rest, and single-variant enums. The order probe set
// shares `core_destructuring_assign_order`: discarded and rest
// elements drop after the binding moves and before the first
// target place evaluates, and braced fields write in the order
// they are written in the target.
struct G(i32);

impl Drop for G {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

struct Pair(G, G);

struct S {
    x: G,
    y: G,
    z: G,
}

struct Wrap<T> {
    val: T,
    extra: T,
}

struct GenPair<T>(T, T);

enum E {
    Only(G, G),
}

enum F {
    Only { x: G, y: G },
}

struct Outer {
    s: S,
    n: G,
}

struct Trio(G, G, G);

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

fn wrap() -> Wrap<G> {
    println!("call wrap");
    Wrap { val: make(20), extra: make(21) }
}

fn tuple_struct() {
    println!("-- tuple struct");
    let mut a = [G(10), G(11)];
    Pair(a[f()], _) = Pair(make(20), make(21));
    println!("done {}", a[0].0);
    Pair(_, a[g()]) = Pair(make(30), make(31));
    println!("done {}", a[1].0);
}

fn braced() {
    println!("-- braced");
    let mut a = [G(10), G(11)];
    // Written order in the target is y first, then x.
    S { y: a[g()], x: a[f()], z: _ } = S { x: make(20), y: make(21), z: make(22) };
    println!("done {} {}", a[0].0, a[1].0);
}

fn braced_rest() {
    println!("-- braced rest");
    let mut a = [G(10), G(11)];
    S { x: a[f()], .. } = S { x: make(20), y: make(21), z: make(22) };
    println!("done {}", a[0].0);
}

fn shorthand() {
    println!("-- shorthand");
    let mut x = G(9);
    S { x, y: _, z: _ } = S { x: make(20), y: make(21), z: make(22) };
    println!("done {}", x.0);
}

fn place_rhs() {
    println!("-- struct place rhs");
    let mut a = [G(10), G(11)];
    let s = S { x: make(20), y: make(21), z: make(22) };
    println!("s made");
    S { x: a[f()], .. } = s;
    println!("reuse {} {}", s.y.0, s.z.0);
    println!("done {}", a[0].0);
}

fn nested_struct() {
    println!("-- nested struct");
    let mut a = [G(10), G(11)];
    let o = Outer { s: S { x: make(20), y: make(21), z: make(22) }, n: make(23) };
    println!("o made");
    Outer { s: S { x: a[f()], .. }, .. } = o;
    println!("reuse {} {}", o.s.y.0, o.n.0);
    println!("done {}", a[0].0);
}

fn generics() {
    println!("-- generics");
    let mut a = [G(10), G(11)];
    GenPair(a[f()], _) = GenPair(make(20), make(21));
    println!("done {}", a[0].0);
    Wrap { val: a[f()], .. } = wrap();
    println!("done {}", a[0].0);
}

fn single_variant() {
    println!("-- single variant");
    let mut a = [G(10), G(11)];
    E::Only(a[f()], _) = E::Only(make(20), make(21));
    println!("done {}", a[0].0);
    F::Only { x: a[f()], .. } = F::Only { x: make(30), y: make(31) };
    println!("done {}", a[0].0);
}

fn tuple_struct_rest() {
    println!("-- tuple struct rest");
    let mut a = [G(10), G(11)];
    Trio(a[f()], ..) = Trio(make(20), make(21), make(22));
    println!("done {}", a[0].0);
    Trio(.., a[g()]) = Trio(make(30), make(31), make(32));
    println!("done {}", a[1].0);
}

fn main() {
    tuple_struct();
    braced();
    braced_rest();
    shorthand();
    place_rhs();
    nested_struct();
    generics();
    single_variant();
    tuple_struct_rest();
}
