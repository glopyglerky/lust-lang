// expect: L0305
struct D(u8);

struct S {
    f: D,
    g: u8,
}

fn main() {
    let a = [S { f: D(1), g: 1 }, S { f: D(2), g: 2 }];
    let x = a[1].f;
    println!("x {}", x.0);
}
