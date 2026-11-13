// The fixed form of tests/reject/glob_order_refetch_drops_vis.rs: an explicit import
// sets the binding, so the glob order does not matter.
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
    pub use crate::shapes::Square;
    use crate::prelude::*;
}

fn main() {
    println!("{}", geometry::Square::sides());
}
