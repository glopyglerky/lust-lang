// expect: L0802
// rustc E0603: a `pub(super)` re-export is visible only in the parent.
mod a {
    pub mod m2 {
        macro_rules! k {
            () => {
                2u32
            };
        }
        pub(super) use k;
    }
    pub fn g() -> u32 {
        m2::k!()
    }
}
mod x {
    pub fn f() -> u32 {
        crate::a::m2::k!()
    }
}
fn main() {
    println!("{} {}", x::f(), a::g());
}
