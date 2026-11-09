use std::mem::size_of;

const LO: u8 = 10;
const HI: u8 = 20;

const fn n() -> usize {
    4
}
const N: usize = n();

fn mk() -> [u8; N] {
    [1; N]
}

fn f(x: u8) -> u8 {
    match x {
        LO..=HI => 1,
        u8::MAX => 2,
        0..LO => 3,
        _ => 0,
    }
}

fn g(x: i128) -> i8 {
    match x {
        i128::MIN..=-1 => -1,
        0 => 0,
        1..=i128::MAX => 1,
    }
}

fn h(x: u8) -> u8 {
    match x {
        0..=127 => 1,
        128..=255 => 2,
    }
}

struct S {
    a: u32,
    b: u8,
}

fn main() {
    let a = [0u8; size_of::<S>()];
    let b = [1u8; size_of::<u32>() * 2];
    let c = [2u8; n()];
    println!("{} {} {} {:?}", a.len(), b.len(), c.len(), mk());
    println!("{} {} {} {}", f(15), f(25), f(255), f(3));
    println!("{} {}", g(-5), g(170141183460469231731687303715884105727));
    println!("{} {}", h(3), h(200));
}
