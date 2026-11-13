use std::cell::RefCell;
thread_local! {
    static V: RefCell<Vec<u8>> = RefCell::new(vec![1, 2]);
}
fn main() {
    V.with_borrow_mut(|v| v.push(3));
    V.with_borrow(|v| println!("{:?} {}", v, v.len()));
}
