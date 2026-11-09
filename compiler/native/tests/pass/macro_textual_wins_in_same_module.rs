// A `macro_rules!` definition written in the module of a `use` of another
// macro with its name shadows it; `pub(crate) use k;` before a later
// local definition re-exports the outer one.
macro_rules! k {
    () => {
        1u32
    };
}
mod m2 {
    macro_rules! k {
        () => {
            2u32
        };
    }
    pub(crate) use k;
}
mod m {
    pub(crate) use k;
    macro_rules! k {
        () => {
            3u32
        };
    }
    pub fn f() -> u32 {
        k!()
    }
}
use m2::k as other;
fn main() {
    println!("{} {} {} {}", k!(), other!(), m::k!(), m::f());
}
