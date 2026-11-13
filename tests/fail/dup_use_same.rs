// expect: L0106
mod m {
    pub fn f() {}
}
use m::f;
use m::f;
fn main() {
    f();
}
