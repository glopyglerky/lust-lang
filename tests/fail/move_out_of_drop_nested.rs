// expect: L0305
// rustc E0509: the move goes through `p.s`, whose type implements
// `Drop`, even though `P` itself does not.
struct D(u8);

struct S {
    a: D,
}

impl Drop for S {
    fn drop(&mut self) {
        println!("drop S");
    }
}

struct P {
    s: S,
    b: D,
}

fn main() {
    let p = P { s: S { a: D(1) }, b: D(2) };
    let x = p.s.a;
    println!("{} {}", x.0, p.b.0);
}
