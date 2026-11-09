mod a {
    pub use crate::b::*;
    pub struct X;
}
mod b {
    pub use crate::a::*;
}
fn main() {
    let _x = b::X;
    println!("ok");
}
