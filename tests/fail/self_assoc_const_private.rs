// expect: L0510
mod a {
    pub struct S {
        pub x: i32,
    }
    impl S {
        const C: i32 = 3;
    }
}
impl a::S {
    fn other(&self) -> i32 {
        Self::C + self.x
    }
}
fn main() {
    println!("{}", a::S { x: 1 }.other());
}
