// The same item through a public glob and a private prelude glob: rustc keeps the
// widest visibility in every order, so lust accepts it (see L0509).
mod shapes {
    pub struct Square;
    impl Square {
        pub fn sides() -> u32 {
            4
        }
    }
}

mod prelude {
    pub use crate::shapes::*;
}

mod geometry {
    pub use crate::shapes::*;
    use crate::prelude::*;
}

fn main() {
    println!("{}", geometry::Square::sides());
}
