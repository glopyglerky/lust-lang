mod a {
    pub use crate::b::*;
    pub fn unrelated() {}
}
mod b {
    pub use crate::a::*;
}
fn main() {
    b::unrelated();
    println!("ok");
}
