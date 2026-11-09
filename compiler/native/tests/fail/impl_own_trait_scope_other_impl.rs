// expect: L0104
// rustc E0599: a trait impl puts its trait in scope only inside that impl, not in another impl of the same type.
mod t {
    pub trait Tr {
        fn b(&self) -> u32 {
            1
        }
    }
    impl Tr for u32 {}
}
mod m {
    pub struct S;
    impl crate::t::Tr for S {}
    impl S {
        pub fn z(&self) -> u32 {
            5u32.b()
        }
    }
}
fn main() {
    println!("{}", m::S.z());
}
