use std::sync::{Arc, Weak};
fn main() {
    let w: Weak<String>;
    {
        let a = Arc::new(String::from("gone"));
        w = Arc::downgrade(&a);
        println!("{}", w.upgrade().unwrap());
    }
    println!("{}", w.upgrade().is_none());
}
