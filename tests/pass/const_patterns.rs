const MAX: i64 = 7;
const NEG: i64 = -3;
const A: i64 = 1 + 2;
const LO: i64 = 2;
const HI: i64 = 5;
const S: &str = "hi";
const T: bool = true;
const F: bool = false;
const C: char = 'x';

enum E {
    X,
    Y,
}
use E::*;

mod m {
    pub const S: i64 = 7;
}

fn pick(x: i64) -> i64 {
    match x {
        MAX => 1,
        NEG => 2,
        A => 3,
        LO..=HI => 4,
        _ => 0,
    }
}

fn main() {
    println!("{} {} {} {}", pick(7), pick(-3), pick(3), pick(4));
    println!("{}", pick(9));
    match "hi" {
        S => println!("s"),
        _ => println!("o"),
    }
    match false {
        T => println!("t"),
        F => println!("f"),
    }
    match 'x' {
        C => println!("c"),
        _ => println!("o"),
    }
    match E::Y {
        X => println!("x"),
        Y => println!("y"),
    }
    match 7 {
        m::S => println!("ms"),
        _ => println!("o"),
    }
    match (3, 1) {
        (A, _) => println!("pair"),
        _ => println!("o"),
    }
    match 4 {
        ..=HI => println!("le"),
        _ => println!("gt"),
    }
    match 5i64 {
        i64::MAX => println!("max"),
        _ => println!("small"),
    }
    match i64::MAX {
        i64::MAX => println!("max"),
        _ => println!("small"),
    }
}
