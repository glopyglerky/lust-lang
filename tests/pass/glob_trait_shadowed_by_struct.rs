// A local struct and an explicit import shadow a glob-imported trait of the
// same name (ts-rust checker::EmitResolver vs printer::EmitResolver), in
// the impl header, in an inherent impl, and in a `Type::method` call.
mod printer {
    pub mod types {
        pub trait EmitResolver {
            fn get(&self) -> i32;
        }
    }
    pub use types::*;
}
mod checker {
    pub mod types {
        pub struct T;
    }
    pub mod p1 {
        use crate::prelude::*;
        pub struct EmitResolver {
            pub v: i32,
        }
        impl crate::printer::EmitResolver for EmitResolver {
            fn get(&self) -> i32 {
                EmitResolver::get(self) + 1
            }
        }
    }
    pub mod p2 {
        use crate::checker::p1::EmitResolver;
        use crate::prelude::*;
        impl EmitResolver {
            pub fn get(&self) -> i32 {
                self.v * 2
            }
        }
    }
    pub use p1::*;
}
mod prelude {
    pub use crate::checker::types;
    pub use crate::checker::*;
    pub use crate::printer::*;
}
fn main() {
    let r = checker::p1::EmitResolver { v: 21 };
    use printer::EmitResolver as _;
    println!("{} {}", r.get(), crate::printer::EmitResolver::get(&r));
}
