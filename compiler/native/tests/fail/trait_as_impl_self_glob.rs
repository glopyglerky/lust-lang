// expect: L0102
// rustc E0782: a glob brings the trait `EmitResolver` (no struct of that name is in scope), and a trait is not a type. lust names the trait by its source path.
mod printer {
    pub mod types {
        pub trait EmitResolver {
            fn get(&self) -> i32;
        }
    }
    pub use types::*;
}
mod checker {
    use crate::printer::*;
    impl EmitResolver {
        pub fn twice(&self) -> i32 {
            2
        }
    }
}
fn main() {}
