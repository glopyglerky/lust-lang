// expect: L0510
mod m {
    pub struct S {
        pub a: i32,
        b: i32,
    }
    pub fn mk() -> S {
        S { a: 1, b: 2 }
    }
    impl S {
        pub fn b(&self) -> i32 {
            self.b
        }
    }
}
fn main() {
    let s = m::mk();
    let t = m::S { a: 5, ..s };
    println!("{} {}", t.a, t.b());
}
