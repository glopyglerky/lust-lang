// expect: L0510
mod a {
    pub struct S {
        x: i32,
    }
    impl S {
        pub fn new() -> S {
            S { x: 1 }
        }
    }
}
fn get<T>(s: &a::S, t: T) -> i32 {
    let _ = &t;
    let f = || s.x;
    f()
}
fn main() {
    let s = a::S::new();
    println!("{}", get(&s, 3));
}
