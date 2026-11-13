// expect: L0306
// `replace` stores its argument in the thread's value, which is 'static.
use std::cell::RefCell;
thread_local! {
    static S: RefCell<Vec<&'static str>> = RefCell::new(Vec::new());
}
fn main() {
    let s = String::from("x");
    let _old = S.replace(vec![&s]);
}
