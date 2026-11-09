// expect: L0102
mod m {
    use std::rc::Rc;
    pub fn mk() -> Rc<u8> {
        Rc::new(3)
    }
}
fn main() {
    let r: Rc<u8> = m::mk();
    println!("{}", r);
}
