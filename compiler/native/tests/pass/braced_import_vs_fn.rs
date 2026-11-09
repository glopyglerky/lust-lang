mod m {
    pub struct S {
        pub x: i64,
    }
}
use m::S;
fn S() -> i64 {
    7
}
fn main() {
    let s = S { x: 3 };
    println!("{} {}", S(), s.x);
}
