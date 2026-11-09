// expect: L0305
// rustc E0509: `S` implements `Drop`, so no field may move out of it.
struct D(u8);

struct S {
    a: D,
}

impl Drop for S {
    fn drop(&mut self) {
        println!("drop S");
    }
}

fn main() {
    let s = S { a: D(1) };
    let x = s.a;
    println!("{}", x.0);
}
