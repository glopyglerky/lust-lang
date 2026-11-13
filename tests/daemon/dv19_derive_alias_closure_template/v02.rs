// A generic fn with a closure names S only through a type alias; Copy goes out (cold L0301) and back.
const ROUTE: u8 = 0;
#[derive(Clone, Debug)]
pub struct S {
    pub x: u32,
}
type T2 = S;
mod m {
    pub use super::S as Q;
}
fn g<T>(_t: T) -> u32 { let f = || 1; let s = T2 { x: 1 }; let a = s; let b = s; a.x + b.x + f() }
fn main() {
    let _r = ROUTE;
    println!("{}", g(1u8));
}
