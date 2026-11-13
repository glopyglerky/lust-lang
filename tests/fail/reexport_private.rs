// expect: L0510
mod a {
    fn f() -> i32 {
        1
    }
    pub use self::f as g;
}
fn main() {
    println!("{}", a::g());
}
