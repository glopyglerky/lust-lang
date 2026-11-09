// The borrow an impl Trait value captures ends when the value is dead
// or dropped, so later moves and mutations of the source are fine.
use std::fmt::Display;
fn lbl(t: &String) -> impl Display { format!("<{}>", t) }
fn main() {
    let mut s = String::from("x");
    {
        let d = lbl(&s);
        println!("{}", d);
    }
    s.push('y');
    println!("{}", lbl(&s));
    s.push('z');
    let owned = format!("{}", lbl(&s));
    drop(s);
    println!("{}", owned);
}
