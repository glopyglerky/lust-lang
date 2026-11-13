// expect: L0733
// A borrow of the guarded value cannot leave `with_borrow`: the closure
// argument is higher-ranked (rustc: lifetime may not live long enough).
use std::cell::RefCell;
thread_local! {
    static N: RefCell<Vec<String>> = RefCell::new(vec!["a".to_string()]);
}
fn main() {
    let s: &str = N.with_borrow(|v| v[0].as_str());
    println!("{}", s);
}
