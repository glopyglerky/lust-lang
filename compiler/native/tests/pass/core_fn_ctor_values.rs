#[derive(Debug)]
enum E {
    A(u8, u16),
    B,
}
#[derive(Debug)]
struct P(i32, i32);
fn main() {
    let f = E::A;
    println!("{:?}", f(1, 2));
    let mk = P;
    let mk2 = mk;
    let p = mk2(3, 4);
    println!("{:?}", p);
    let one = if true { P } else { P };
    println!("{:?}", one(5, 6));
}
