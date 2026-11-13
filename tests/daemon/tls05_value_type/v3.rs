use std::cell::RefCell;
thread_local! {
    static V: RefCell<Vec<String>> = RefCell::new(vec!["a".to_string()]);
}
fn main() {
    V.with_borrow_mut(|v| v.push("b".to_string()));
    V.with_borrow(|v| println!("{:?} {}", v, v.len()));
}
