// expect: L0506
// rustc E0432: `crate::m::n::*` inside `n` names `n` itself.
mod m {
    pub mod n {
        #[allow(unused_imports)]
        use crate::m::n::*;
        pub fn f() -> u8 {
            1
        }
    }
}
fn main() {
    println!("{}", m::n::f());
}
