// The const evaluator reads a lone name in a pattern as the checker does:
// the root's const `N` and unit struct `y` are not in scope in `m`, so `N`
// and `y` there bind new locals (rustc). A block const and an imported
// const still compare as path patterns, and so does the prelude's `None`.
#[allow(dead_code)]
const N: i32 = 3;
#[allow(non_camel_case_types, dead_code)]
#[derive(Clone, Copy)]
struct y;
mod m {
    #[allow(non_snake_case, unreachable_patterns)]
    pub const fn g(x: i32) -> i32 {
        match x {
            N => N * 10,
        }
    }
    pub const fn h(x: i32) -> i32 {
        match x {
            y => y + 1,
        }
    }
    pub const fn k(x: i32) -> i32 {
        const K: i32 = 4;
        match x {
            K => 1,
            _ => 0,
        }
    }
    pub mod i {
        use crate::N;
        pub const fn g(x: i32) -> i32 {
            match x {
                N => 7,
                _ => 8,
            }
        }
    }
    pub const fn o(x: Option<i32>) -> i32 {
        match x {
            None => -1,
            Some(v) => v,
        }
    }
    pub const V: [i32; 8] = [g(4), h(4), k(4), k(5), i::g(3), i::g(4), o(None), o(Some(9))];
}
fn main() {
    println!("{:?}", m::V);
    println!("{} {} {} {} {} {}", m::g(4), m::h(4), m::k(4), m::k(5), m::i::g(3), m::i::g(4));
}
