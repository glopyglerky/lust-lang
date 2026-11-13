// expect: L0509
// Valid Rust in this order. `geometry` gets `Square` through a public glob and a private
// glob of `prelude`, whose own binding can still widen and be re-sent. In 16 of the 96
// orders of these modules and imports rustc drops the public binding (E0603), so lust
// rejects the class (subset rule).
mod shapes {
    pub struct Square;
    impl Square {
        pub fn sides() -> u32 {
            4
        }
    }
}

mod extra {
    pub use crate::shapes::*;
}

mod prelude {
    pub use crate::shapes::*;
    pub(crate) use crate::extra::*;
}

mod geometry {
    pub use crate::shapes::*;
    use crate::prelude::*;
}

fn main() {
    println!("{}", geometry::Square::sides());
}
