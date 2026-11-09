#[derive(PartialEq, Eq)]
enum C {
    A,
    B,
    X(u8),
}
const CA: C = C::A;
const NONE8: Option<u8> = None;
fn f(c: C) -> u32 {
    match c {
        CA => 0,
        C::B => 1,
        C::X(_) => 2,
    }
}
fn g(o: Option<u8>) -> u32 {
    match o {
        NONE8 => 0,
        Some(_) => 1,
    }
}
fn main() {
    println!("{} {} {}", f(C::A), f(C::B), f(C::X(3)));
    println!("{} {}", g(None), g(Some(9)));
    const BF: f64 = 2.5;
    match 2.5f64 {
        BF => println!("bf"),
        _ => println!("o"),
    }
}
