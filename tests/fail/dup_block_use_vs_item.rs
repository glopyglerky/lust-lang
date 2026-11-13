// expect: L0106
mod m {
    pub fn f() {}
}
fn main() {
    use m::f;
    fn f() {}
    f();
}
