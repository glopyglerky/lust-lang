// expect: L0510
mod a {
    pub struct S {
        pub shown: i32,
        x: i32,
    }
    pub fn mk() -> S {
        S { shown: 1, x: 2 }
    }
}
fn pick<T>(s: a::S, t: T) -> i32 {
    let _ = &t;
    let a::S { x, .. } = s;
    x
}
fn main() {
    println!("{}", pick(a::mk(), 3));
}
