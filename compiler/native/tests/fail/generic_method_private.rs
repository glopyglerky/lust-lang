// expect: L0510
mod a {
    pub struct S;
    impl S {
        fn secret(&self) -> i32 {
            9
        }
        pub fn new() -> S {
            S
        }
    }
}
fn call<T>(s: &a::S, t: T) -> i32 {
    let _ = &t;
    s.secret()
}
fn main() {
    let s = a::S::new();
    println!("{}", call(&s, 3));
}
