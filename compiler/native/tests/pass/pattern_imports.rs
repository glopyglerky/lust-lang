mod m {
    pub const K: i32 = 3;
    pub const L: i32 = 4;
}
#[derive(Debug, Clone, Copy)]
enum E {
    A,
    B(u8),
    C { x: u8 },
}
#[derive(Debug, Clone, Copy)]
pub struct U;
use E::*;
use m::K;
use m::*;

fn f(x: i32) -> i32 {
    match x {
        K => 1,
        L => 2,
        m::K => 3,
        _ => 0,
    }
}

fn g(e: E) -> u8 {
    match e {
        A => 1,
        B(n) => n,
        C { x } => x + 100,
    }
}

fn h(x: u8) -> u8 {
    match x {
        u8::MAX => 1,
        u8::MIN => 2,
        _ => 0,
    }
}

const V: E = C { x: 3 };

fn main() {
    let u = U;
    match u {
        U => println!("u"),
    }
    println!("{} {} {} {}", f(3), f(4), f(5), f(K));
    println!("{} {} {}", g(A), g(B(3)), g(C { x: 1 }));
    println!("{} {} {}", h(255), h(0), h(7));
    println!("{:?}", V);
    let x = 9i32;
    if let K = x {
        println!("three");
    } else {
        println!("not three");
    }
    let o = Some(3i32);
    if let Some(K) = o {
        println!("some three");
    }
}
