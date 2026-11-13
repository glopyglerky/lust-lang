// expect: L0510
mod a {
    pub struct S {
        x: i32,
    }
    impl S {
        pub fn new() -> S {
            S { x: 1 }
        }
        fn get(&self) -> i32 {
            self.x
        }
    }
}
impl a::S {
    fn other(&self) -> i32 {
        Self::get(self)
    }
}
fn main() {
    println!("{}", a::S::new().other());
}
