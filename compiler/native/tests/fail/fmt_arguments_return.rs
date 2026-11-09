// expect: L0307
// A `format_args!` value with an argument borrows its function's
// arguments array, even when the argument is a static (rustc E0515).
use std::fmt;

static S: &str = "s";

fn f() -> fmt::Arguments<'static> {
    format_args!("{}", S)
}

fn main() {
    println!("{}", f());
}
