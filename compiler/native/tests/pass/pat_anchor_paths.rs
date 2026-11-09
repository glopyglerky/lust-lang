// Patterns that start with a module anchor: `crate::`, `super::`,
// `self::` (W-BENCH2 bug 8). Tuple and unit variants, a struct
// pattern, a unit path, and range bounds by const path.
mod k {
    pub enum Kind {
        A(u32),
        B,
    }
    pub struct S {
        pub x: u32,
        pub y: u32,
    }
    pub const LO: u32 = 2;
    pub const HI: u32 = 5;
    pub const ONE: u32 = 1;
}
mod inner {
    pub fn g(x: super::k::Kind) -> u32 {
        match x {
            super::k::Kind::A(n) => n,
            super::k::Kind::B => 0,
        }
    }
    pub fn h(s: crate::k::S) -> u32 {
        match s {
            crate::k::S { x, .. } => x,
        }
    }
}
fn f(x: k::Kind) -> u32 {
    match x {
        crate::k::Kind::A(n) => n,
        crate::k::Kind::B => 0,
    }
}
fn r(n: u32) -> u32 {
    match n {
        crate::k::ONE => 10,
        crate::k::LO..=crate::k::HI => 20,
        _ => 30,
    }
}
fn main() {
    let s = match k::Kind::A(3) {
        self::k::Kind::A(n) => n,
        _ => 0,
    };
    println!("{} {} {} {} {}", f(k::Kind::A(4)), inner::g(k::Kind::B), inner::h(k::S { x: 7, y: 1 }), r(1) + r(3) + r(9), s);
}
