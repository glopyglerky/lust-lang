// expect: L0507
// rustc E0659: `k` is a `macro_rules!` name from the parent module and a
// different macro imported in this module.
macro_rules! k {
    () => {
        1u32
    };
}
mod user {
    use crate::m2::k;
    pub fn f() -> u32 {
        k!()
    }
}
mod m2 {
    macro_rules! k {
        () => {
            2u32
        };
    }
    pub(crate) use k;
}
fn main() {
    println!("{}", user::f());
}
