// expect: L0106
mod a {
    pub fn b() {}
    pub fn c() {}
}
fn main() {
    use a::b;
    use a::c as b;
}
