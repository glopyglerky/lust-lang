// expect: L0510
mod a {
    pub struct S {
        pub shown: i32,
        x: i32,
    }
    impl S {
        pub fn new() -> S {
            S { shown: 1, x: 2 }
        }
    }
}
fn set<T>(t: T) -> i32 {
    let _ = &t;
    let mut s = a::S::new();
    s.x = 5;
    s.shown
}
fn main() {
    println!("{}", set(3));
}
