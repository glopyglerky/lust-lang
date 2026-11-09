// expect: L0510
mod a {
    pub struct S {
        pub shown: i32,
        x: i32,
    }
}
fn make<T>(t: T) -> i32 {
    let _ = &t;
    let s = a::S { shown: 1, x: 2 };
    s.shown
}
fn main() {
    println!("{}", make(3));
}
