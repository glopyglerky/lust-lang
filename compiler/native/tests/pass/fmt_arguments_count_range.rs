// rustc checks a width taken from an argument when the `format_args!`
// value is made, so an out-of-range width panics at the macro, before
// any use (R9).
use std::fmt;

fn show(a: fmt::Arguments) -> String {
    format!("{}", a)
}

fn main() {
    let w = std::env::args().count() * 70000;
    println!("{}", show(format_args!("{:1$}", 1, 3)).len());
    let a = format_args!("{:w$}", 1);
    println!("made");
    println!("{}", show(a).len());
}
