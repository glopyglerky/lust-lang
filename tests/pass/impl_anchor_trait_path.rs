// Path-qualified trait refs in impl headers (`impl crate::a::T
// for V`), `self::` and `super::` forms, and an inherent impl whose
// Self type is written by a `crate::` path (W-BENCH2 bug 7).
mod a {
    pub trait T {
        fn n(&self) -> u32;
    }
    pub struct S;
    pub mod b {
        pub struct U;
        impl super::super::a::T for U {
            fn n(&self) -> u32 {
                3
            }
        }
    }
}
struct V;
impl crate::a::T for V {
    fn n(&self) -> u32 {
        1
    }
}
impl self::a::T for a::S {
    fn n(&self) -> u32 {
        2
    }
}
impl a::S {
    fn extra(&self) -> u32 {
        40
    }
}
mod sub {
    impl crate::a::S {
        pub fn more(&self) -> u32 {
            50
        }
    }
}
use a::T as _;
fn main() {
    let s = a::S;
    println!("{} {} {} {} {}", V.n(), s.n(), a::b::U.n(), s.extra(), s.more());
}
