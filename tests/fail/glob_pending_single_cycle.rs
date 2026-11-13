// expect: L0506
// `s::n` imports `m::n`, which `m` can only get through its glob of `s`,
// whose glob binding of `n` the pending import hides (rustc E0432).
mod s {
    pub use crate::m::n;
    pub use crate::t::*;
}
mod m {
    pub use crate::s::*;
}
mod t {
    pub fn n() -> u32 {
        7
    }
}
fn main() {
    println!("{}", s::n());
}
