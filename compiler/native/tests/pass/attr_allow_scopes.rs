#![allow(dead_code)]
//! Scope-level `#[allow(..)]` forms reach every function in scope:
//! a crate inner attribute, an inline module inner attribute, an
//! outer attribute on a module, an impl inner attribute, and an
//! outer attribute on an impl. The overflowing functions are never
//! called; the lint is what is tested.
#![allow(arithmetic_overflow)]

fn crate_first() -> u8 {
    let a: u8 = 255;
    a + 1
}

fn crate_second() -> u8 {
    let a: u8 = 255;
    a + 1
}

mod inner_mod {
    #![allow(arithmetic_overflow)]
    pub fn a() -> u8 {
        let x: u8 = 255;
        x + 1
    }
    pub fn b() -> u8 {
        let x: u8 = 255;
        x + 1
    }
}

#[allow(arithmetic_overflow)]
mod outer_mod {
    pub fn a() -> u8 {
        let x: u8 = 255;
        x + 1
    }
}

struct S;

impl S {
    #![allow(arithmetic_overflow)]
    fn a() -> u8 {
        let x: u8 = 255;
        x + 1
    }
    fn b() -> u8 {
        let x: u8 = 255;
        x + 1
    }
}

struct T;

#[allow(arithmetic_overflow)]
impl T {
    fn a() -> u8 {
        let x: u8 = 255;
        x + 1
    }
}

fn main() {
    let _ = crate_first;
    let _ = crate_second;
    let _ = inner_mod::a;
    let _ = inner_mod::b;
    let _ = outer_mod::a;
    println!("ok");
}
