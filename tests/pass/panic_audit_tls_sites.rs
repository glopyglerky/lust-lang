// A thread-local RefCell's key methods panic inside std's
// thread/local.rs (with_borrow_mut here), not at the caller.
// panic: RefCell already borrowed
use std::cell::RefCell;
thread_local! {
    static S: RefCell<String> = RefCell::new(String::from("a"));
}
fn main() {
    S.with_borrow_mut(|s| s.push('b'));
    S.with_borrow(|s| {
        println!("{}", s);
        S.with_borrow_mut(|t| t.push('c'));
    });
}
