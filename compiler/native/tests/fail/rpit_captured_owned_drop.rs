// expect: L0311
// rustc E0505: in edition 2024 the impl Trait value captures the borrow of `s`, even though its hidden type (String) holds no reference (hunt2 rpit_captured_lifetime_hidden_owned).
use std::fmt::Display;
fn lbl(t: &String) -> impl Display { format!("<{}>", t) }
fn main() {
    let s = String::from("x");
    let d = lbl(&s);
    drop(s);
    println!("{}", d);
}
