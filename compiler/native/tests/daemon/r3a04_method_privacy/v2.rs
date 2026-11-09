mod m {
    pub struct S;
    impl S {
        fn f() -> u32 {
            3
        }
    }
}
fn main() {
    println!("{}", m::S::f());
}
