// expect: L0102
// rustc E0433: a glob of a sibling module does not bring its private std
// import.
mod p {
    use std::rc::Rc;
    pub fn get() -> Rc<u8> {
        Rc::new(1)
    }
}
use p::*;
fn main() {
    let r: Rc<u8> = get();
    println!("{}", r);
}
