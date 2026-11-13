// expect: L0733
// A `Ref` guard borrows the key's value: it cannot leave `with`.
use std::cell::RefCell;
thread_local! {
    static N: RefCell<String> = RefCell::new(String::from("a"));
}
fn main() {
    let g = N.with(|n| n.borrow());
    println!("{}", g);
}
