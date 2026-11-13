use std::cell::RefCell;
struct D(u8);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
thread_local! {
    static K: RefCell<D> = const { RefCell::new(D(1)) };
}
fn main() {
    K.set(D(2));
    K.with_borrow(|d| println!("value {}", d.0));
}
