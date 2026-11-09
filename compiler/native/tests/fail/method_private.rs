// expect: L0510
mod m {
    pub struct S;
    impl S {
        fn f(&self) -> i32 {
            1
        }
        pub fn new() -> S {
            S
        }
    }
}
fn main() {
    let s = m::S::new();
    println!("{}", s.f());
}
