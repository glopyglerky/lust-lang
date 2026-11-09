mod m {
    pub struct S;
    impl S {
        pub fn f() -> u32 {
            3
        }
    }
}
fn main() {
    println!("{}", m::S::f());
}
