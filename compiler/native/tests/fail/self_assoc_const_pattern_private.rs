// expect: L0510
mod a {
    pub struct S;
    impl S {
        const C: i32 = 3;
    }
}
impl a::S {
    fn hit(n: i32) -> i32 {
        match n {
            Self::C => 1,
            _ => 0,
        }
    }
}
fn main() {
    println!("{}", a::S::hit(3));
}
