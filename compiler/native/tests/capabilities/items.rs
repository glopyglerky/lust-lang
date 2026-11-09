// capability: items: fn
// capability: items: struct (named fields, tuple fields, or unit)
// capability: items: enum (unit, tuple, and struct variants)
// capability: items: trait (default methods, associated types and consts, generic traits, supertraits)
// capability: items: impl (inherent, trait, and generic impls)
// capability: items: use (paths, renames, globs, and groups)
// capability: items: mod (inline modules and `mod name;` files)
// capability: items: const and static items
// capability: items: type aliases
// capability: items: macro_rules (definitions and calls, #[macro_export])
// expect: ok

mod shapes {
    pub struct Named {
        pub a: u32,
    }
    pub struct Tuple(pub u32);
    pub struct Unit;
    pub enum E {
        A,
        B(u32),
        C { x: u32 },
    }
    #[macro_export]
    macro_rules! twice {
        ($e:expr) => {
            $e * 2
        };
    }
}

use shapes::{E, Named as N, Tuple, Unit};
use std::collections::*;

trait Shape: std::fmt::Debug {
    type Out;
    const SIDES: u32;
    fn area(&self) -> Self::Out;
    fn sides(&self) -> u32 {
        Self::SIDES
    }
}

trait Scale<T> {
    fn scale(&self, by: T) -> T;
}

#[derive(Debug)]
struct Sq(u32);

impl Shape for Sq {
    type Out = u32;
    const SIDES: u32 = 4;
    fn area(&self) -> u32 {
        self.0 * self.0
    }
}

impl Scale<u32> for Sq {
    fn scale(&self, by: u32) -> u32 {
        self.0 * by
    }
}

struct Wrap<T>(T);

impl<T: Copy> Wrap<T> {
    fn get(&self) -> T {
        self.0
    }
}

const K: u32 = 3;
static NAME: &str = "items";
type Pair = (u32, u32);

fn val(e: &E) -> u32 {
    match e {
        E::A => 0,
        E::B(v) => *v,
        E::C { x } => *x,
    }
}

fn main() {
    let n = N { a: 1 };
    let Tuple(t) = Tuple(2);
    let _u = Unit;
    let p: Pair = (n.a, t);
    let s = Sq(K);
    let mut m = HashMap::new();
    m.insert(1, val(&E::C { x: 5 }) + val(&E::B(1)) + val(&E::A));
    println!("{} {:?} {} {} {} {} {} {}", NAME, p, s.area(), s.sides(), s.scale(2), Wrap(7u8).get(), m[&1], twice!(4));
}
