mod program { pub struct S(pub u8); }
mod rootprelude { pub use crate::program::*; }
mod frontend {
    pub mod prelude {
        pub use crate::deep1::*;
        pub use crate::rootprelude::*;
        pub use crate::deep1::S;
    }
    pub mod user {
        use crate::frontend::prelude::*;
        pub fn get() -> u16 { S(300).0 }
    }
}
mod deep1 { pub use crate::deep2::*; }
mod deep2 { pub use crate::deep3::*; }
mod deep3 { pub struct S(pub u16); }
fn main() { println!("{}", frontend::user::get()); }
