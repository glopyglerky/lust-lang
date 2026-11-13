// panic: RefCell already borrowed
// `set` on a live RefCell key writes through `borrow_mut`, which panics
// while a shared borrow is out.
use std::cell::RefCell;

thread_local! {
    static S: RefCell<String> = RefCell::new(String::from("a"));
}

fn main() {
    S.with_borrow(|s| {
        println!("borrowed {}", s);
        S.set(String::from("b"));
    });
}
