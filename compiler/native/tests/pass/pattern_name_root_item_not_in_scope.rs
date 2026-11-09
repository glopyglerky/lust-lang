// A lone name in a pattern or parameter is a fresh binding unless the
// module binds it: the root's const `N` and unit struct `f` are not in
// scope in `m`, so `N` and `f` there bind new locals (rustc). Through an
// import (`use crate::N`, a glob, `use E::*`) they are path patterns.
#[allow(dead_code)]
const N: i32 = 3;
#[allow(non_camel_case_types, dead_code)]
struct f;
#[allow(dead_code)]
enum E {
    A,
    B,
}
mod m {
    #[allow(non_snake_case, unreachable_patterns)]
    pub fn g(x: i32) -> i32 {
        match x {
            N => N * 10,
            _ => 0,
        }
    }
    pub fn h() -> i32 {
        let f = 5;
        f + 1
    }
    #[allow(non_snake_case)]
    pub fn p(N: i32) -> i32 {
        N * 2
    }
    pub fn path(x: i32) -> i32 {
        match x {
            crate::N => 7,
            _ => 0,
        }
    }
    pub mod k {
        use crate::N;
        pub fn h(x: i32) -> i32 {
            match x {
                N => 5,
                _ => 6,
            }
        }
    }
}
mod glob {
    use super::*;
    pub fn g(x: i32) -> i32 {
        match x {
            N => 1,
            _ => 2,
        }
    }
    pub fn v(e: E) -> i32 {
        use E::*;
        match e {
            A => 10,
            B => 20,
        }
    }
}
fn main() {
    println!("{} {} {} {}", m::g(3), m::g(4), m::h(), m::p(8));
    println!("{} {} {} {}", m::path(3), m::path(4), m::k::h(3), m::k::h(4));
    println!("{} {} {}", glob::g(3), glob::g(4), glob::v(E::B));
}
