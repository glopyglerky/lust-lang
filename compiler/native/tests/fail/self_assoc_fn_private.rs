// expect: L0510
mod a {
    pub struct S {
        pub x: i32,
    }
    impl S {
        fn make() -> S {
            S { x: 1 }
        }
    }
}
impl a::S {
    fn other() -> i32 {
        Self::make().x
    }
}
fn main() {
    println!("{}", a::S::other());
}
