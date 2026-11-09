// A module's glob binding of a name stays hidden from its glob importers
// while a single import of that name there is pending (ts-rust EmitHost).
// `prelude` first gets the struct `EmitHost` through `program::*`; its
// `pub use crate::printer::EmitHost` resolves later. `printer::p2` globs
// both `prelude` and `printer` privately: without the rule it saw the
// struct and the trait at once, and that private ambiguity reached
// `printer` through `pub use p2::*`.
mod prelude {
    pub use crate::printer::*;
    pub use crate::program::*;
    pub use crate::printer::EmitHost;
}
mod printer {
    // The globs come before the modules, as with out-of-line `mod` files:
    // `printer` binds the trait before `p2` reads it.
    pub use p2::*;
    pub use types::*;
    pub mod types {
        use crate::prelude::*;
        pub trait EmitHost {
            fn blocked(&self) -> bool;
        }
        pub fn helper() -> u32 {
            1
        }
    }
    pub mod p2 {
        use crate::prelude::*;
        use crate::printer::*;
        pub fn p2f() -> u32 {
            helper() + 1
        }
    }
}
mod program {
    use crate::prelude::*;
    pub struct EmitHost;
    impl crate::printer::EmitHost for EmitHost {
        fn blocked(&self) -> bool {
            true
        }
    }
    pub fn mk() -> EmitHost {
        EmitHost
    }
}
mod emitter {
    use crate::prelude::*;
    pub fn go() -> bool {
        let h = mk();
        crate::printer::EmitHost::blocked(&h)
    }
}
fn main() {
    println!("{} {}", emitter::go(), printer::p2f());
}
