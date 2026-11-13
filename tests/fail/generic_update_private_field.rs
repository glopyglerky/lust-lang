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
fn upd<T>(s: a::S, t: T) -> i32 {
    let _ = &t;
    let s2 = a::S { shown: 9, ..s };
    s2.shown
}
fn main() {
    println!("{}", upd(a::mk(), 3));
}
