mod m {
    pub fn f() -> i64 {
        9
    }
}
use m::*;
use m::f;
fn main() {
    println!("{}", f());
}
