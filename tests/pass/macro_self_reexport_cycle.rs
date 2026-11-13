// `macro_rules! m` plus `pub(crate) use m;` in one module, with a
// parent glob that re-exports the macro back into it (W-BENCH3 bug 8).
mod ast {
    pub mod synthetic {
        use crate::ast::*;
        macro_rules! with_data {
            ($n:expr) => { $n * 10 };
        }
        pub(crate) use with_data;
        pub fn one() -> u32 { 1 }
    }
    pub mod fields {
        use crate::ast::synthetic::with_data;
        pub fn f() -> u32 { with_data!(super::one()) }
    }
    pub use synthetic::*;
}
fn main() { println!("{}", ast::fields::f()); }
