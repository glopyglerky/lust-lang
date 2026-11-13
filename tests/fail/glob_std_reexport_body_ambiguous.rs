// expect: L0507
// rustc E0659: in a fn body, two globs bring a std re-export and a crate
// struct of one name.
mod a {
    pub use std::rc::Rc;
}
mod b {
    pub struct Rc(pub u8);
}
fn main() {
    use a::*;
    use b::*;
    let r = Rc::new(1u8);
    println!("{}", r);
}
