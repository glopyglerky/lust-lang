// expect: L0104
mod a {
    pub struct S;
    pub trait Tr {
        fn tm(&self) -> i32;
    }
    impl Tr for S {
        fn tm(&self) -> i32 {
            21
        }
    }
    impl S {
        pub fn new() -> S {
            S
        }
    }
}
fn call<T>(s: &a::S, t: T) -> i32 {
    let _ = &t;
    s.tm()
}
fn main() {
    let s = a::S::new();
    println!("{}", call(&s, 3));
}
