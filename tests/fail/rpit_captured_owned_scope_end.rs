// expect: L0311
// rustc E0505: a generic definer at `&String` captures the borrow, and the value may use it when it drops at scope end (hunt2 rpit_captured_lifetime_hidden_owned).
use std::fmt::Display;
fn lbl<T: Display>(t: T) -> impl Display { format!("<{}>", t) }
fn main() {
    let s = String::from("x");
    let d = lbl(&s);
    println!("{}", d);
    drop(s);
}
