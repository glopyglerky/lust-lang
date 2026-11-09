// expect: L0305
// rustc E0509: a by-value binding in a pattern moves the payload out
// of an enum that implements `Drop`.
struct D(u8);

enum E {
    A(D),
    B,
}

impl Drop for E {
    fn drop(&mut self) {
        println!("drop E");
    }
}

fn main() {
    let e = E::A(D(1));
    match e {
        E::A(d) => println!("{}", d.0),
        E::B => {}
    }
}
