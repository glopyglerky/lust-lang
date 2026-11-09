mod a {
    pub fn helper() -> u32 {
        1
    }
}
mod b {
    pub struct X;
}
fn f() -> u32 {
    use crate::a::*;
    use crate::b::*;
    let _x: X = X;
    helper()
}
fn main() {
    println!("{}", f());
}
