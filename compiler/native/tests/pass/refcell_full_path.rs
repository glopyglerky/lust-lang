// A full std path builds a RefCell where only another module imports it.
mod m {
    use std::cell::RefCell;
    pub struct S {
        pub c: RefCell<u8>,
    }
}
fn main() {
    let s = m::S { c: std::cell::RefCell::new(1) };
    *s.c.borrow_mut() += 1;
    println!("{}", s.c.borrow());
}
