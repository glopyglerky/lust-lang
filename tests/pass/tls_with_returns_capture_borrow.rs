// A `with` closure may return a borrow of what it captures (not of its
// argument): the result outlives the call.
use std::cell::RefCell;
thread_local! {
    static N: Vec<String> = vec!["a".to_string()];
    static R: RefCell<Vec<u8>> = RefCell::new(vec![1]);
}
fn main() {
    let local = String::from("x");
    let r: &String = N.with(|_| &local);
    let bytes = vec![9u8, 8];
    let b = R.with_borrow(|_| &bytes[1]);
    println!("{} {}", r, b);
}
