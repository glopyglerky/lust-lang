#[derive(Debug, Clone, Copy)]
enum E {
    A(i32),
    B { z: u8, w: i64 },
    C,
}

const X: u128 = 1 << 127;
const Y: u128 = X - 1 + X;
const GT: bool = X > 5;
const D: u128 = X / 3;
const S: u128 = X >> 100;
const M: u128 = u128::MAX / 2 + 1;
const NEG: i8 = -128 / 2;
const V: E = E::B { z: 9, w: -4 };
static SV: E = E::B { z: 1, w: 2 };
const O: Option<E> = Some(E::B { z: 2, w: 3 });

const fn fact(n: u64) -> u64 {
    let mut r = 1;
    let mut i = 1;
    while i <= n {
        r *= i;
        i += 1;
    }
    r
}

const fn wide() -> u32 {
    let x = 0xffff_fff0;
    x / 16
}

const F20: u64 = fact(20);
const W: u32 = wide();

#[derive(Debug, Clone, Copy)]
struct P {
    x: i32,
    y: i32,
}
const ORIGIN: P = P { x: 0, y: -1 };
const UP: P = P { x: 5, ..ORIGIN };

fn main() {
    println!("{} {} {} {} {} {}", X, Y, GT, D, S, M);
    println!("{} {:?} {:?} {:?}", NEG, V, SV, O);
    match V {
        E::A(n) => println!("A {}", n),
        E::B { z, w } => println!("B {} {}", z, w),
        E::C => println!("C"),
    }
    let base = E::B { z: 1, w: 2 };
    println!("{:?} {:?} {} {}", base, UP, F20, W);
}
