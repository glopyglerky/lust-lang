// expect: L0510
mod m {
    pub struct S;
    impl S {
        const SEC: i32 = 1;
    }
}
fn g<T>(t: T) -> i32 {
    let _ = &t;
    m::S::SEC
}
fn main() {
    println!("{}", g(3));
}
