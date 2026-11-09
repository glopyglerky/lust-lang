// expect: L0733
// rustc: lifetime may not live long enough, also when the definer is never called (hunt2 rpit_static_bound_hidden_borrow).
use std::fmt::Debug;
fn bad<'a>(v: &'a String) -> impl Debug + 'static { (v, String::new()) }
fn main() {
    println!("ok");
}
